use crate::{
    game::resources::Resources,
    graphics::{Renderer, Vulkan},
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

#[derive(Debug, Clone)]
pub struct UiState {
    pub canvases: Vec<Canvas>,
    pub is_transparent: bool,
}

impl UiState {
    pub fn default_state() -> Self {
        let canvas = Canvas {
            center: Vec2::new(-0.8, -0.8),
            width: 0.2,
            height: 0.1,
            color: Vec4::ONE.with_w(0.5),
            ..Default::default()
        };

        Self {
            canvases: vec![canvas],
            is_transparent: true,
        }
    }

    pub fn is_transparent(&self) -> bool {
        self.is_transparent
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
                "Called render on a GameState object but the gui_pipeline is not initialized in the renderer"
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
            let push_constant = canvas.push_constant();
            let vertex_buffer = canvas.into_vertex_buffer(vulkan.memory_allocator.clone());
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

        secondary_builder.build().unwrap()
    }
}
