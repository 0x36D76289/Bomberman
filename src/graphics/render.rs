use crate::{
    app::App,
    graphics::{vs, window_size_dependent_setup},
};
use vulkano::{
    Validated, VulkanError,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo},
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    pipeline::{Pipeline, PipelineBindPoint},
    swapchain::{SwapchainCreateInfo, SwapchainPresentInfo, acquire_next_image},
    sync::{self, GpuFuture},
};

impl App {
    pub fn draw_frame(&mut self) {
        let rcx = self.rcx.as_mut().unwrap();
        let vulkan = &self.vulkan;
        let window_size = rcx.window.inner_size();

        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        rcx.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if rcx.recreate_swapchain {
            let (new_swapchain, new_images) = rcx
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: window_size.into(),
                    ..rcx.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            rcx.swapchain = new_swapchain;
            (rcx.framebuffers, rcx.pipeline) = window_size_dependent_setup(
                window_size,
                &new_images,
                &rcx.render_pass,
                &vulkan.memory_allocator,
                &rcx.vs,
                &rcx.fs,
            );
            rcx.recreate_swapchain = false;
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(rcx.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    rcx.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            rcx.recreate_swapchain = true;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.15, 0.15, 0.15, 1.0].into()), Some(1f32.into())],
                    ..RenderPassBeginInfo::framebuffer(
                        rcx.framebuffers[image_index as usize].clone(),
                    )
                },
                Default::default(),
            )
            .unwrap()
            .bind_pipeline_graphics(rcx.pipeline.clone())
            .unwrap();

        let aspect_ratio =
            rcx.swapchain.image_extent()[0] as f32 / rcx.swapchain.image_extent()[1] as f32;
        self.camera.set_perspective_projection(0.87, aspect_ratio, 0.1, 10.0);

        for object in self.objects.iter() {
            let uniform_buffer = {
                let uniform_data = vs::Data {
                    world: object.transform.mat4().to_cols_array_2d(),
                    view: self.camera.view_matrix.to_cols_array_2d(),
                    proj: self.camera.projection_matrix.to_cols_array_2d(),
                    color: object.color.to_array()
                };

                let buffer = vulkan.uniform_buffer_allocator.allocate_sized().unwrap();
                *buffer.write().unwrap() = uniform_data;

                buffer
            };

            let layout = &rcx.pipeline.layout().set_layouts()[0];
            let descriptor_set = DescriptorSet::new(
                vulkan.descriptor_set_allocator.clone(),
                layout.clone(),
                [WriteDescriptorSet::buffer(0, uniform_buffer)],
                [],
            )
            .unwrap();

            builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    rcx.pipeline.layout().clone(),
                    0,
                    descriptor_set,
                )
                .unwrap()
                .bind_vertex_buffers(0, object.model.vertex_buffer.clone())
                .unwrap()
                .bind_index_buffer(object.model.index_buffer.clone())
                .unwrap();

            unsafe {
                builder
                    .draw_indexed(object.model.index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }

        builder.end_render_pass(Default::default()).unwrap();

        let command_buffer = builder.build().unwrap();
        let future = rcx
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(vulkan.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                vulkan.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(rcx.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                rcx.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                rcx.recreate_swapchain = true;
                rcx.previous_frame_end = Some(sync::now(vulkan.device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                rcx.previous_frame_end = Some(sync::now(vulkan.device.clone()).boxed());
            }
        }

        // rcx.update_time();
        // rcx.input_state.reset();
        // rcx.window
        //     .set_title(&format!("Scop! fps: {:.2}", rcx.avg_fps()));
    }
}
