use std::{error::Error, sync::Arc, time::Instant};

use crate::{
    app::App,
    graphics::{
        Camera, GameObject, Light, RenderContext, Vulkan,
        systems::{GlobalUbo, game_object_system::GameObjectSystem},
        window_size_dependent_setup,
    },
    input::KeyboardMovementController,
    load_model,
};
use glam::{Vec3, Vec4};
use vulkano::{
    Validated, VulkanError,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    },
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{Pipeline, PipelineBindPoint, graphics::viewport::Viewport},
    swapchain::{
        SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture},
};
use winit::event_loop::{ActiveEventLoop, EventLoop};

pub struct Renderer {
    vulkan: Vulkan,
    rcx: Option<RenderContext>,
    pub game_object_system: Option<GameObjectSystem>,
    image_index: u32,
    acquire_future: Option<SwapchainAcquireFuture>,
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let vulkan = Vulkan::init(event_loop)?;

        Ok(Self {
            vulkan,
            rcx: None,
            game_object_system: None,
            image_index: Default::default(),
            acquire_future: None,
        })
    }

    pub fn init_render_context(&mut self, event_loop: &ActiveEventLoop) {
        self.rcx = Some(RenderContext::init(event_loop, &self.vulkan).unwrap())
    }

    pub fn init_game_object_system(&mut self) {
        self.game_object_system = Some(GameObjectSystem::init(
            &self.vulkan,
            self.rcx.as_ref().unwrap().render_pass.clone(),
            self.rcx.as_ref().unwrap().window.inner_size(),
        ));
    }

    pub fn get_aspect_ration(&self) -> f32 {
        let rcx = self.rcx.as_ref().unwrap();
        rcx.swapchain.image_extent()[0] as f32 / rcx.swapchain.image_extent()[1] as f32
    }

    pub fn get_memory_allocator(&self) -> Arc<StandardMemoryAllocator> {
        self.vulkan.memory_allocator.clone()
    }

    pub fn get_delta_time(&self) -> f32 {
        self.rcx.as_ref().unwrap().time_info.dt
    }

    pub fn recreate_swapchain(&mut self, b: bool) {
        self.rcx.as_mut().unwrap().recreate_swapchain = b;
    }

    pub fn request_redraw(&self) {
        self.rcx.as_ref().unwrap().window.request_redraw();
    }

    pub fn begin_frame(&mut self) -> Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>> {
        let rcx = self.rcx.as_mut().unwrap();

        let window_size = rcx.window.inner_size();

        if window_size.width == 0 || window_size.height == 0 {
            return None;
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
            rcx.framebuffers = window_size_dependent_setup(
                window_size,
                &new_images,
                &rcx.render_pass,
                &self.vulkan.memory_allocator,
            );
            rcx.recreate_swapchain = false;
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(rcx.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    rcx.recreate_swapchain = true;
                    return None;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            rcx.recreate_swapchain = true;
        }

        self.image_index = image_index;
        self.acquire_future = Some(acquire_future);

        let mut builder = AutoCommandBufferBuilder::primary(
            self.vulkan.command_buffer_allocator.clone(),
            self.vulkan.queue.queue_family_index(),
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
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: window_size.into(),
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap();

        Some(builder)
    }

    pub fn end_frame(
        &mut self,
        mut command_buffer: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let rcx = self.rcx.as_mut().unwrap();

        command_buffer.end_render_pass(Default::default()).unwrap();

        let acquire_future = self.acquire_future.take().unwrap();

        let command_buffer = command_buffer.build().unwrap();
        let future = rcx
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.vulkan.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.vulkan.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    rcx.swapchain.clone(),
                    self.image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                rcx.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                rcx.recreate_swapchain = true;
                rcx.previous_frame_end = Some(sync::now(self.vulkan.device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                rcx.previous_frame_end = Some(sync::now(self.vulkan.device.clone()).boxed());
            }
        }
    }

    pub fn render_game_objects(
        &self,
        objects: &Vec<GameObject>,
        global_ubo: GlobalUbo,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let game_object_system = self.game_object_system.as_ref().unwrap();

        game_object_system.render_game_objects(&self.vulkan, objects, global_ubo, command_buffer);
    }

    pub fn update_time(&mut self) {
        let time_info = &mut self.rcx.as_mut().unwrap().time_info;

        time_info.dt = time_info.time.elapsed().as_secs_f32();
        time_info.time = Instant::now();
        time_info.dt_sum += time_info.dt;
        time_info.frame_count += 1.0;

        // calculate the fps every second
        if (time_info.dt_sum > 1.0) {
            time_info.avg_fps = time_info.frame_count / time_info.dt_sum;
            time_info.dt_sum = 0.0;
            time_info.frame_count = 0.0;
        }
    }

    pub fn update_title(&mut self) {
        let rcx = self.rcx.as_ref().unwrap();

        let fps = rcx.time_info.avg_fps;

        let title = format!("Bomberman! fps: {:.0}", fps);
        rcx.window.set_title(&title);
    }
}
