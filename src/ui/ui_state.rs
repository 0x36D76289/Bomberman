use crate::{
    app_state::{AppState, KeyMap},
    game::resources::{ResourceName, Resources},
    graphics::{GuiPush, Renderer, Vulkan},
    input::input::Input,
    ui::canvas::Canvas,
};
use glam::{Vec2, Vec4};
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
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone)]
pub struct UiState {
    pub canvases: Vec<Canvas>,
    pub is_transparent: bool,
}

impl UiState {
    pub fn default_state() -> Self {
        let title = Canvas {
            center: Vec2::new(0.0, -0.3),
            text: Some("BOMBERMAN".to_string()),
            text_color: Some(Vec4::ONE),
            text_size: Some(2.0),
            ..Default::default()
        };

        let text1 = Canvas {
            center: Vec2::new(0.0, 0.2),
            text: Some("Press enter to play!".to_string()),
            text_color: Some(Vec4::ONE),
            text_size: Some(0.8),
            ..Default::default()
        };

        Self {
            canvases: vec![title, text1],
            is_transparent: false,
        }
    }

    pub fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    pub fn tick(
        &mut self,
        _delta_time: f32,
        _inputs: &Vec<Input>,
        keys: &KeyMap,
        _resources: &Resources,
    ) -> (Option<AppState>, u8) {
        match keys.get(&PhysicalKey::Code(KeyCode::Enter)) {
            Some(state) if state.is_pressed() => (None, 1),
            _ => (None, 0),
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

        for canvas in self.canvases.iter() {
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
