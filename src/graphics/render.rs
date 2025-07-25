use crate::{app::App, graphics::window_size_dependent_setup};
use vulkano::{
    Validated, VulkanError,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo},
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

        // let layout = &rcx.pipeline.layout().set_layouts()[0];
        // let descriptor_set = DescriptorSet::new(
        //     vulkan.descriptor_set_allocator.clone(),
        //     layout.clone(),
        //     [],
        //     [],
        // )
        // .unwrap();

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
            .unwrap()
            .bind_vertex_buffers(0, vulkan.vertex_buffer.clone())
            .unwrap();
        unsafe { builder.draw(vulkan.vertex_buffer.len() as u32, 1, 0, 0) }.unwrap();

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
