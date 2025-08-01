use std::{collections::HashSet, hash::RandomState, sync::Arc};

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo},
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
    shader::EntryPoint,
};
use winit::dpi::PhysicalSize;

use crate::{
    app::App,
    graphics::{GameObject, MyVertex, RenderContext, Vulkan, systems::GlobalUbo},
};

pub struct GameObjectSystem {
    pub pipeline: Arc<GraphicsPipeline>,
}

impl GameObjectSystem {
    pub fn init(
        vulkan: &Vulkan,
        render_pass: Arc<RenderPass>,
        window_size: PhysicalSize<u32>,
    ) -> Self {
        GameObjectSystem {
            pipeline: create_pipeline(vulkan, render_pass, window_size),
        }
    }

    pub fn render_game_objects(
        &self,
        vulkan: &Vulkan,
        objects: &Vec<GameObject>,
        global_ubo: GlobalUbo,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let uniform_buffer = {
            let buffer = vulkan.uniform_buffer_allocator.allocate_sized().unwrap();
            *buffer.write().unwrap() = global_ubo;

            buffer
        };

        let layout = &self.pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new(
            vulkan.descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();

        command_buffer
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap();

        for object in objects.iter() {
            if object.model.is_none() {
                continue;
            }

            let model = object.model.as_ref().unwrap();

            let push_constant = vs::Push {
                model_matrix: object.transform.mat4().to_cols_array_2d(),
                normal_matrix: object.transform.normal_matrix().to_cols_array_2d(),
                color: object.color.to_array(),
            };

            command_buffer
                .push_constants(self.pipeline.layout().clone(), 0, push_constant)
                .unwrap()
                .bind_vertex_buffers(0, model.vertex_buffer.clone())
                .unwrap()
                .bind_index_buffer(model.index_buffer.clone())
                .unwrap();

            unsafe {
                command_buffer
                    .draw_indexed(model.index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }
    }
}

fn create_pipeline(
    vulkan: &Vulkan,
    render_pass: Arc<RenderPass>,
    window_size: PhysicalSize<u32>,
) -> Arc<GraphicsPipeline> {
    let vertex_shader = vs::load(vulkan.device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();
    let fragment_shader = fs::load(vulkan.device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();

    let vertex_input_state = MyVertex::per_vertex().definition(&vertex_shader).unwrap();
    let stages = [
        PipelineShaderStageCreateInfo::new(vertex_shader.clone()),
        PipelineShaderStageCreateInfo::new(fragment_shader.clone()),
    ];
    let layout = PipelineLayout::new(
        vulkan.device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(vulkan.device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(render_pass, 0).unwrap();

    GraphicsPipeline::new(
        vulkan.device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            viewport_state: Some(Default::default()),
            input_assembly_state: Some(InputAssemblyState::default()),
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/vertex.glsl"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/fragment.glsl"
    }
}
