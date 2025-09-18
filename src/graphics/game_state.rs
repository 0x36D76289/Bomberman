use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        CopyImageToBufferInfo, RenderingAttachmentInfo, RenderingAttachmentResolveInfo,
        RenderingInfo, SubpassContents,
    },
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    format::ClearValue,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{Pipeline, PipelineBindPoint, graphics::viewport::Viewport},
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
};

use crate::{
    game::{Camera, game_state::GameState, resources::Resources},
    graphics::{GamePush, GlobalUbo, Renderer, Vulkan, renderer::RENDER_RES_RATIO},
};

impl Renderer {
    pub fn render_game(
        &mut self,
        vulkan: &Vulkan,
        resources: &Resources,
        state: &GameState,
        image_index: u32,
        is_first: bool,
    ) {
        self.shadow_pass(vulkan, state);
        self.game_pass(vulkan, resources, state, is_first);
        self.postprocess_pass(vulkan, image_index, is_first);
    }

    fn game_pass(
        &mut self,
        vulkan: &Vulkan,
        resources: &Resources,
        state: &GameState,
        is_first: bool,
    ) {
        let (pipeline, command_buffer, rcx) = match (
            self.game_pipeline.as_ref(),
            self.command_buffer.as_mut(),
            self.rcx.as_ref(),
        ) {
            (Some(pipeline), Some(command_buffer), Some(rcx)) => {
                (pipeline.clone(), command_buffer, rcx)
            }
            (None, _, _) => panic!("Pipeline is not initialized"),
            (_, None, _) => panic!("Command buffer is not started"),
            (_, _, None) => panic!("Render context is not initialized"),
        };

        let window_size: [u32; 2] = rcx.swapchain.image_extent();
        let game_resolution = [
            window_size[0] / RENDER_RES_RATIO[0],
            window_size[1] / RENDER_RES_RATIO[1],
        ];

        let global_ubo = {
            let aspect_ratio = window_size[0] as f32 / window_size[1] as f32;
            state.create_ubo(aspect_ratio)
        };

        let rendering_info = {
            let mut color_attachment = RenderingAttachmentInfo::image_view(rcx.color_image.clone());
            color_attachment.store_op = AttachmentStoreOp::Store;
            color_attachment.load_op = AttachmentLoadOp::Load;
            if is_first {
                color_attachment.load_op = AttachmentLoadOp::Clear;
                color_attachment.clear_value = Some(ClearValue::Float([0.0, 0.0, 0.0, 0.0]));
            }
            let mut depth_attachment = RenderingAttachmentInfo::image_view(rcx.depth_image.clone());
            depth_attachment.load_op = AttachmentLoadOp::Clear;
            depth_attachment.store_op = AttachmentStoreOp::Store;
            depth_attachment.clear_value = Some(1f32.into());
            RenderingInfo {
                color_attachments: vec![Some(color_attachment)],
                depth_attachment: Some(depth_attachment),
                contents: SubpassContents::Inline,
                ..Default::default()
            }
        };

        command_buffer.begin_rendering(rendering_info).unwrap();

        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: [game_resolution[0] as f32, game_resolution[1] as f32],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
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
            resources.textures.len() as u32,
            [
                WriteDescriptorSet::buffer(0, uniform_buffer),
                WriteDescriptorSet::sampler(1, self.sampler.clone()),
                WriteDescriptorSet::image_view_array(2, 0, resources.textures.clone()),
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

        for object in state.objects_to_render() {
            let push_constant = GamePush {
                model_matrix: object.transform.mat4().to_cols_array_2d(),
                normal_matrix: object.transform.normal_matrix().to_cols_array_2d(),
                color: object.color.to_array(),
                tex_index: object.texture.unwrap_or(-1),
            };

            command_buffer
                .push_constants(pipeline.layout().clone(), 0, push_constant)
                .unwrap()
                .bind_vertex_buffers(0, object.model.vertex_buffer.clone())
                .unwrap()
                .bind_index_buffer(object.model.index_buffer.clone())
                .unwrap();

            unsafe {
                command_buffer
                    .draw_indexed(object.model.index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }

        command_buffer.end_rendering().unwrap();
    }

    fn shadow_pass(&mut self, vulkan: &Vulkan, state: &GameState) {
        let (pipeline, command_buffer, rcx) = match (
            self.shadows_pipeline.as_ref(),
            self.command_buffer.as_mut(),
            self.rcx.as_ref(),
        ) {
            (Some(pipeline), Some(command_buffer), Some(rcx)) => {
                (pipeline.clone(), command_buffer, rcx)
            }
            (None, _, _) => panic!("Pipeline is not initialized"),
            (_, None, _) => panic!("Command buffer is not started"),
            (_, _, None) => panic!("Render context is not initialized"),
        };

        let window_size: [u32; 2] = rcx.swapchain.image_extent();
        let game_resolution = [
            window_size[0] / RENDER_RES_RATIO[0],
            window_size[1] / RENDER_RES_RATIO[1],
        ];

        let global_ubo = {
            let aspect_ratio = window_size[0] as f32 / window_size[1] as f32;
            state.create_ubo(aspect_ratio)
        };

        let rendering_info = {
            let depth_attachment = RenderingAttachmentInfo {
                load_op: AttachmentLoadOp::Clear,
                store_op: AttachmentStoreOp::Store,
                clear_value: Some(1f32.into()),
                ..RenderingAttachmentInfo::image_view(rcx.depth_image.clone())
            };
            RenderingInfo {
                depth_attachment: Some(depth_attachment),
                contents: SubpassContents::Inline,
                ..Default::default()
            }
        };

        command_buffer.begin_rendering(rendering_info).unwrap();

        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: [game_resolution[0] as f32, game_resolution[1] as f32],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap();

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
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap();

        for object in state.objects_to_render() {
            let push_constant = GamePush {
                model_matrix: object.transform.mat4().to_cols_array_2d(),
                normal_matrix: object.transform.normal_matrix().to_cols_array_2d(),
                color: object.color.to_array(),
                tex_index: object.texture.unwrap_or(-1),
            };

            command_buffer
                .push_constants(pipeline.layout().clone(), 0, push_constant)
                .unwrap()
                .bind_vertex_buffers(0, object.model.vertex_buffer.clone())
                .unwrap()
                .bind_index_buffer(object.model.index_buffer.clone())
                .unwrap();

            unsafe {
                command_buffer
                    .draw_indexed(object.model.index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }

        command_buffer.end_rendering().unwrap();
    }

    fn postprocess_pass(&mut self, vulkan: &Vulkan, image_index: u32, is_first: bool) {
        let (pipeline, command_buffer, rcx) = match (
            self.post_process_pipeline.as_ref(),
            self.command_buffer.as_mut(),
            self.rcx.as_ref(),
        ) {
            (Some(pipeline), Some(command_buffer), Some(rcx)) => {
                (pipeline.clone(), command_buffer, rcx)
            }
            (None, _, _) => panic!("Pipeline is not initialized"),
            (_, None, _) => panic!("Command buffer is not started"),
            (_, _, None) => panic!("Render context is not initialized"),
        };

        let rendering_info_info = {
            let mut color_attachment =
                RenderingAttachmentInfo::image_view(rcx.images[image_index as usize].clone());
            color_attachment.store_op = AttachmentStoreOp::Store;
            color_attachment.load_op = AttachmentLoadOp::Load;
            if is_first {
                color_attachment.load_op = AttachmentLoadOp::Clear;
                color_attachment.clear_value = Some(ClearValue::Float([0.0, 0.0, 0.0, 0.0]));
            }
            RenderingInfo {
                color_attachments: vec![Some(color_attachment)],
                layer_count: 1,
                contents: SubpassContents::Inline,
                ..Default::default()
            }
        };

        command_buffer.begin_rendering(rendering_info_info).unwrap();
        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: rcx.window.inner_size().into(),
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let layout = &pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new(
            vulkan.descriptor_set_allocator.clone(),
            layout.clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                rcx.color_image.clone(),
                self.sampler.clone(),
            )],
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

        unsafe {
            command_buffer.draw(3, 1, 0, 0).unwrap();
        }

        command_buffer.end_rendering().unwrap();
    }
}
