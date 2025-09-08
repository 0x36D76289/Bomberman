use crate::{
    app_state::{AppState, KeyMap},
    game::resources::{ResourceName, Resources},
    graphics::{GuiPush, Renderer, Vulkan},
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{button::Button, canvas::Canvas, game_settings::UIGameSettings, utils::GetRatio},
};
use glam::Vec4;
use std::sync::Arc;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferInheritanceInfo,
        CommandBufferInheritanceRenderPassType, CommandBufferInheritanceRenderingInfo,
        CommandBufferUsage, SecondaryAutoCommandBuffer,
    },
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    pipeline::{Pipeline, PipelineBindPoint, graphics::viewport::Viewport},
};

/// What UI is in use
#[derive(Debug, Copy, Clone)]
pub enum UIPage {
    MainMenu,
    Pause,
    GameSettings(UIGameSettings),
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub canvases: Vec<Canvas>,
    pub buttons: Vec<Button>,
    pub is_transparent: bool,
    pub selected: usize,
    pub page: UIPage,
}

impl UiState {
    pub fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    fn select_button(&mut self, target: usize) {
        let mut target = target;
        if target >= self.buttons.len() {
            target = 0;
        }

        self.buttons[self.selected].toggle();
        self.buttons[target].toggle();
        self.selected = target;
    }

    /// Returns true if confirm button is used
    pub fn button_inputs(&mut self, inputs: &Vec<Input>) -> bool {
        if self.buttons.len() != 0 {
            if inputs.menu_up() == InputState::Pressed {
                self.select_button(self.buttons[self.selected].neighbors.up);
            }
            if inputs.menu_down() == InputState::Pressed {
                self.select_button(self.buttons[self.selected].neighbors.down);
            }
            if inputs.menu_left() == InputState::Pressed {
                self.select_button(self.buttons[self.selected].neighbors.left);
            }
            if inputs.menu_right() == InputState::Pressed {
                self.select_button(self.buttons[self.selected].neighbors.right);
            }
        }
        inputs.menu_confirm() == InputState::Pressed
    }

    pub fn tick(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
    ) -> (Option<AppState>, u8) {
        match self.page {
            UIPage::MainMenu => self.main_menu_tick(keys),
            UIPage::Pause => self.pause_tick(inputs, resources),
            UIPage::GameSettings(_) => self.game_settings_tick(delta, inputs, resources),
        }
    }

    pub fn render(
        &self,
        vulkan: &Vulkan,
        renderer: &Renderer,
        resources: &Resources,
    ) -> Arc<SecondaryAutoCommandBuffer> {
        let pipeline = match renderer.gui_pipeline.as_ref() {
            Some(pipeline) => pipeline.clone(),
            None => panic!(
                "Called render on a UiState object but the gui_pipeline is not initialized in the renderer"
            ),
        };

        let format = renderer.rcx().swapchain.image_format();

        let inheritance_rendering_info = CommandBufferInheritanceRenderingInfo {
            color_attachment_formats: vec![Some(format)],
            ..Default::default()
        };

        let mut secondary_builder = AutoCommandBufferBuilder::secondary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(CommandBufferInheritanceRenderPassType::BeginRendering(
                    inheritance_rendering_info,
                )),
                ..Default::default()
            },
        )
        .unwrap();

        secondary_builder
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: renderer.rcx().window.inner_size().into(),
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let layout = &pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new_variable(
            vulkan.descriptor_set_allocator.clone(),
            layout.clone(),
            resources.textures.len() as u32,
            [
                WriteDescriptorSet::sampler(0, renderer.sampler.clone()),
                WriteDescriptorSet::image_view_array(1, 0, resources.textures.clone()),
            ],
            [],
        )
        .unwrap();

        secondary_builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap();

        let canvases = self.canvases.iter();
        let button_canvaces = {
            let mut ret = Vec::new();
            for canvas in self
                .buttons
                .iter()
                .map(|b| b.generate_canvases(renderer.window_size().get_ratio()))
                .flatten()
                .flatten()
            {
                ret.push(canvas);
            }
            ret
        };

        for canvas in canvases.chain(button_canvaces.iter()) {
            // draw the canvas
            let vertex_buffer = canvas.into_vertex_buffer(vulkan.memory_allocator.clone());
            let vertex_buffer_len = vertex_buffer.len() as u32;
            let push_constant = canvas.push_constant();

            secondary_builder
                .push_constants(pipeline.layout().clone(), 0, push_constant)
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer)
                .unwrap();

            unsafe {
                secondary_builder.draw(vertex_buffer_len, 1, 0, 0).unwrap();
            }

            // draw the text
            match &canvas.text {
                None => (),
                Some(text) => {
                    let vertex_buffer = renderer.text_renderer.render_str(
                        text,
                        canvas.text_size.unwrap_or(1.0),
                        canvas.center,
                        vulkan.memory_allocator.clone(),
                        renderer.window_size().get_ratio(),
                    );

                    let push_constant = GuiPush {
                        color: canvas.text_color.unwrap_or(Vec4::ONE).into(),
                        tex_index: resources.textures_index[&ResourceName::FontAtlas],
                    };
                    let vertex_buffer_len = vertex_buffer.len() as u32;

                    secondary_builder
                        .push_constants(pipeline.layout().clone(), 0, push_constant)
                        .unwrap()
                        .bind_vertex_buffers(0, vertex_buffer)
                        .unwrap();

                    unsafe {
                        secondary_builder.draw(vertex_buffer_len, 1, 0, 0).unwrap();
                    }
                }
            }
        }

        secondary_builder.build().unwrap()
    }
}
