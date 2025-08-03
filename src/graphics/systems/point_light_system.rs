use std::{collections::HashSet, hash::RandomState, sync::Arc};

use glam::{Mat4, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
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
    graphics::{systems::{GlobalUbo, PointLight}, GameEntity, GameEntityType, MyVertex, RenderContext, Vulkan},
};

#[derive(Debug, Default)]
pub struct PointLightSystem {
    pub pipeline: Option<Arc<GraphicsPipeline>>,
}

impl PointLightSystem {
    pub fn render(
        &self,
        vulkan: &Vulkan,
        entities: &Vec<GameEntity>,
        global_ubo: GlobalUbo,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        if self.pipeline.is_none() {
            panic!("Tried to render game objects but the pipeline is not initialized")
        }

        let pipeline = self.pipeline.as_ref().unwrap();

        let uniform_buffer = {
            let buffer = vulkan.uniform_buffer_allocator.allocate_sized().unwrap();
            *buffer.write().unwrap() = global_ubo;

            buffer
        };

        let layout = &pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new(
            vulkan.descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();

        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap();

        for (entity, light_color) in entities.iter().filter_map(|o| match &o.entity_type {
            GameEntityType::Light { color } => Some((o, color)),
            _ => None
        }) {
            let push_constant = vs::Push {
                position: Vec4::splat(1.0).with_xyz(entity.transform.translation).to_array(),
                color: light_color.to_array(),
                radius: entity.transform.scale.x
            };

            command_buffer
                .push_constants(pipeline.layout().clone(), 0, push_constant)
                .unwrap();

            unsafe {
                command_buffer.draw(6, 1, 0, 0);
            }
        }
    }

    pub fn lights_array(&self, objects: &Vec<GameEntity>) -> ([PointLight; 100], i32) {
        let mut vec = [
            PointLight {
                position: Default::default(),
                color: Default::default()
            };
            100
        ];
        let mut i: usize = 0;
        for (object, light_color) in objects.iter().filter_map(|o| match &o.entity_type {
            GameEntityType::Light { color } => Some((o, color)),
            _ => None
        }) {
            vec[i] = PointLight {
                position: Vec4::splat(1.0).with_xyz(object.transform.translation).to_array(),
                color: light_color.to_array()
            };
            i += 1;
        }
        (vec, i as i32)
    }

    pub fn create_pipeline(
        &mut self,
        vulkan: &Vulkan,
        render_pass: Arc<RenderPass>,
        window_size: PhysicalSize<u32>,
    ) {
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
    
        self.pipeline = Some(GraphicsPipeline::new(
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
        .unwrap());
    }
}


pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/point_light.vert"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/point_light.frag"
    }
}
