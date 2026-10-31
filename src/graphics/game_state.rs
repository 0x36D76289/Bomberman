use crate::{
    game::{game_state::GameState, resources::Resources},
    graphics::{
        GamePush, GuiVertex, Renderer, StateRenderInfo, Vulkan,
        draw::{draw, draw_indexed, present_rendering, vertex_buffer, viewport},
        renderer::{
            Resolution,
            animation_vs::{AnimationPush, JointsUbo},
            postprocess_fs::PostPush,
        },
    },
};
use vulkano::{
    command_buffer::{RenderingAttachmentInfo, RenderingInfo, SubpassContents},
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    format::ClearValue,
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
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
        self.game_pass(vulkan, resources, state);
        self.postprocess_pass(vulkan, &state.render_info, image_index, is_first);
    }

    fn game_pass(&mut self, vulkan: &Vulkan, resources: &Resources, state: &GameState) {
        let pipeline = self.game_pipeline.clone().expect("game pipeline");
        let animation_pipeline = self.animation_pipeline.clone().expect("animation pipeline");
        let rcx = self.rcx.as_ref().expect("render context");
        let command_buffer = self.command_buffer.as_mut().expect("command buffer");

        let window_size: [u32; 2] = rcx.swapchain.image_extent();

        command_buffer
            .begin_rendering({
                let mut color_attachment =
                    RenderingAttachmentInfo::image_view(rcx.color_image.clone());
                color_attachment.store_op = AttachmentStoreOp::Store;
                color_attachment.load_op = AttachmentLoadOp::Clear;
                color_attachment.clear_value = Some(ClearValue::Float([0.0, 0.0, 0.0, 0.0]));

                let mut depth_attachment =
                    RenderingAttachmentInfo::image_view(rcx.depth_image.clone());
                depth_attachment.load_op = AttachmentLoadOp::Clear;
                depth_attachment.store_op = AttachmentStoreOp::Store;
                depth_attachment.clear_value = Some(1f32.into());
                RenderingInfo {
                    color_attachments: vec![Some(color_attachment)],
                    depth_attachment: Some(depth_attachment),
                    contents: SubpassContents::Inline,
                    ..Default::default()
                }
            })
            .unwrap();

        let viewport = viewport(rcx.game_resolution.resolution(window_size));
        let uniform_buffer = vulkan.uniform_buffer_allocator.allocate_sized().unwrap();
        *uniform_buffer.write().unwrap() =
            state.create_ubo(window_size[0] as f32 / window_size[1] as f32);
        let global_set = |pipeline: &GraphicsPipeline| {
            DescriptorSet::new_variable(
                vulkan.descriptor_set_allocator.clone(),
                pipeline.layout().set_layouts()[0].clone(),
                resources.textures.len() as u32,
                [
                    WriteDescriptorSet::buffer(0, uniform_buffer.clone()),
                    WriteDescriptorSet::sampler(1, self.game_sampler.clone()),
                    WriteDescriptorSet::image_view_array(2, 0, resources.textures.clone()),
                ],
                [],
            )
            .unwrap()
        };

        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(0, [viewport.clone()].into_iter().collect())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                global_set(&pipeline),
            )
            .unwrap();

        for object in state.objects_to_render() {
            command_buffer
                .push_constants(
                    pipeline.layout().clone(),
                    0,
                    GamePush {
                        model_matrix: object.transform.mat4().to_cols_array_2d(),
                        normal_matrix: object.transform.normal_matrix().to_cols_array_2d(),
                        color: object.color.to_array(),
                        tex_index: object.texture.unwrap_or(-1),
                    },
                )
                .unwrap()
                .bind_vertex_buffers(0, object.model.vertex_buffer.clone())
                .unwrap()
                .bind_index_buffer(object.model.index_buffer.clone())
                .unwrap();

            draw_indexed(command_buffer, object.model.index_buffer.len() as u32);
        }

        command_buffer
            .bind_pipeline_graphics(animation_pipeline.clone())
            .unwrap()
            .set_viewport(0, [viewport].into_iter().collect())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                animation_pipeline.layout().clone(),
                0,
                global_set(&animation_pipeline),
            )
            .unwrap();

        for object in state.animated_objects_to_render() {
            let joints_buffer = vulkan
                .uniform_buffer_allocator
                .allocate_sized::<JointsUbo>()
                .unwrap();
            object.mesh.joint_transforms_to_ubo_buffer(&joints_buffer);

            command_buffer
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    animation_pipeline.layout().clone(),
                    1,
                    DescriptorSet::new(
                        vulkan.descriptor_set_allocator.clone(),
                        animation_pipeline.layout().set_layouts()[1].clone(),
                        [WriteDescriptorSet::buffer(0, joints_buffer)],
                        [],
                    )
                    .unwrap(),
                )
                .unwrap()
                .push_constants(
                    animation_pipeline.layout().clone(),
                    0,
                    AnimationPush {
                        model_matrix: object.transform.mat4().to_cols_array_2d(),
                        normal_matrix: object.transform.normal_matrix().to_cols_array_2d(),
                        color: object.color.to_array(),
                        tex_index: object.texture.unwrap_or(-1),
                        tint_whites: object.tint_whites as i32,
                    },
                )
                .unwrap()
                .bind_vertex_buffers(0, (*object.mesh.vertex_buffer).clone())
                .unwrap()
                .bind_index_buffer((*object.mesh.index_buffer).clone())
                .unwrap();

            draw_indexed(command_buffer, object.mesh.index_buffer.len() as u32);
        }

        command_buffer.end_rendering().unwrap();
    }

    fn postprocess_pass(
        &mut self,
        vulkan: &Vulkan,
        render_info: &StateRenderInfo,
        image_index: u32,
        is_first: bool,
    ) {
        let pipeline = self
            .post_process_pipeline
            .clone()
            .expect("post process pipeline");
        let rcx = self.rcx.as_ref().expect("render context");
        let command_buffer = self.command_buffer.as_mut().expect("command buffer");

        command_buffer
            .begin_rendering(present_rendering(rcx, image_index, is_first))
            .unwrap();
        command_buffer
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [viewport(rcx.swapchain.image_extent())]
                    .into_iter()
                    .collect(),
            )
            .unwrap();

        command_buffer
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                DescriptorSet::new(
                    vulkan.descriptor_set_allocator.clone(),
                    pipeline.layout().set_layouts()[0].clone(),
                    [WriteDescriptorSet::image_view_sampler(
                        0,
                        rcx.color_image.clone(),
                        self.game_sampler.clone(),
                    )],
                    [],
                )
                .unwrap(),
            )
            .unwrap()
            .push_constants(
                pipeline.layout().clone(),
                0,
                PostPush {
                    pixel: matches!(rcx.game_resolution, Resolution::Pixel(_)) as i32,
                },
            )
            .unwrap();

        let top_left = render_info.top_left_coord;
        let bottom_right = render_info.bottom_right_coord;
        let top_right = [bottom_right[0], top_left[1]];
        let bottom_left = [top_left[0], bottom_right[1]];

        command_buffer
            .bind_vertex_buffers(
                0,
                vertex_buffer(
                    vulkan,
                    [
                        (top_right, [1.0, 0.0]),
                        (top_left, [0.0, 0.0]),
                        (bottom_left, [0.0, 1.0]),
                        (bottom_left, [0.0, 1.0]),
                        (bottom_right, [1.0, 1.0]),
                        (top_right, [1.0, 0.0]),
                    ]
                    .into_iter()
                    .map(|(position, uv)| GuiVertex { position, uv }),
                ),
            )
            .unwrap();

        draw(command_buffer, 6);

        command_buffer.end_rendering().unwrap();
    }
}
