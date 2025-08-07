use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::{DescriptorSet, WriteDescriptorSet, layout::DescriptorBindingFlags},
    image::{
        sampler::{BorderColor, Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
        view::ImageView,
    },
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
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
};

use crate::{
    game::Entity,
    graphics::{GlobalUbo, MyVertex, Vulkan},
};

#[derive(Debug, Default)]
pub struct EntityRenderSystem {
    pipeline: Option<Arc<GraphicsPipeline>>,
    sampler: Option<Arc<Sampler>>,
}

impl EntityRenderSystem {
    pub fn render_game_objects(
        &self,
        vulkan: &Vulkan,
        enities: &Vec<Entity>,
        textures: &Vec<Arc<ImageView>>,
        global_ubo: GlobalUbo,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        if self.pipeline.is_none() {
            panic!("Tried to render game objects but the pipeline is not initialized")
        }

        let pipeline = self.pipeline.as_ref().unwrap();

        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap();

        let uniform_buffer = {
            let buffer = vulkan.uniform_buffer_allocator.allocate_sized().unwrap();
            *buffer.write().unwrap() = global_ubo;

            buffer
        };

        let layout = &pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new_variable(
            vulkan.descriptor_set_allocator.clone(),
            layout.clone(),
            textures.len() as u32,
            [
                WriteDescriptorSet::buffer(0, uniform_buffer),
                WriteDescriptorSet::sampler(1, self.sampler.as_ref().unwrap().clone()),
                WriteDescriptorSet::image_view_array(2, 0, textures.clone()),
            ],
            [],
        )
        .unwrap();

        command_buffer
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap();

        for entity in enities
            .iter()
            .filter(|e| e.physics.is_some() && e.model.is_some())
        {
            let transform = entity.physics.unwrap().transform;
            let model = entity.model.as_ref().unwrap();

            let push_constant = vs::Push {
                model_matrix: transform.mat4().to_cols_array_2d(),
                normal_matrix: transform.normal_matrix().to_cols_array_2d(),
                color: entity.color.unwrap_or_default().to_array(),
                tex_index: entity.texture.unwrap_or(-1),
            };

            command_buffer
                .push_constants(pipeline.layout().clone(), 0, push_constant)
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

    pub fn create_pipeline(&mut self, vulkan: &Vulkan, render_pass: Arc<RenderPass>) {
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
        let layout = {
            let mut layout_create_info =
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages);

            let binding = layout_create_info.set_layouts[0]
                .bindings
                .get_mut(&2)
                .unwrap();
            binding.binding_flags |= DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT;
            binding.descriptor_count = 100;

            PipelineLayout::new(
                vulkan.device.clone(),
                layout_create_info
                    .into_pipeline_layout_create_info(vulkan.device.clone())
                    .unwrap(),
            )
            .unwrap()
        };

        let subpass = Subpass::from(render_pass, 0).unwrap();

        self.pipeline = Some(
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
            .unwrap(),
        );

        self.sampler = Some(
            Sampler::new(
                vulkan.device.clone(),
                SamplerCreateInfo {
                    mag_filter: Filter::Nearest,
                    min_filter: Filter::Nearest,
                    address_mode: [SamplerAddressMode::ClampToBorder; 3],
                    border_color: BorderColor::FloatOpaqueWhite,
                    ..Default::default()
                },
            )
            .unwrap(),
        );
    }
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/game_object.vert"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/game_object.frag"
    }
}
