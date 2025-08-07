use crate::graphics::{TimeInfo, Vulkan};
use std::{sync::Arc, time::Instant};
use vulkano::{
    Validated, VulkanError,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    },
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage, view::ImageView},
    memory::allocator::AllocationCreateInfo,
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{
        Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo,
        acquire_next_image,
    },
    sync::{self, GpuFuture},
};
use winit::{event_loop::ActiveEventLoop, window::Window};

pub struct Renderer {
    rcx: Option<RenderContext>,
    image_index: u32,
    acquire_future: Option<SwapchainAcquireFuture>,
}

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub time_info: TimeInfo,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            rcx: None,
            image_index: 0,
            acquire_future: None,
        }
    }

    pub fn init_render_context(&mut self, event_loop: &ActiveEventLoop, vulkan: &Vulkan) {
        self.rcx = Some(RenderContext::init(event_loop, &vulkan).unwrap())
    }

    pub fn get_rcx(&self) -> &RenderContext {
        self.rcx.as_ref().unwrap()
    }

    pub fn get_aspect_ration(&self) -> f32 {
        let rcx = self.rcx.as_ref().unwrap();
        rcx.swapchain.image_extent()[0] as f32 / rcx.swapchain.image_extent()[1] as f32
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

    pub fn begin_frame(
        &mut self,
        vulkan: &Vulkan,
    ) -> Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>> {
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
            rcx.framebuffers = {
                let depth_buffer = ImageView::new_default(
                    Image::new(
                        vulkan.memory_allocator.clone(),
                        ImageCreateInfo {
                            image_type: ImageType::Dim2d,
                            format: Format::D32_SFLOAT,
                            extent: new_images[0].extent(),
                            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT
                                | ImageUsage::TRANSIENT_ATTACHMENT,
                            ..Default::default()
                        },
                        AllocationCreateInfo::default(),
                    )
                    .unwrap(),
                )
                .unwrap();

                new_images
                    .iter()
                    .map(|image| {
                        let view = ImageView::new_default(image.clone()).unwrap();

                        Framebuffer::new(
                            rcx.render_pass.clone(),
                            FramebufferCreateInfo {
                                attachments: vec![view, depth_buffer.clone()],
                                ..Default::default()
                            },
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>()
            };
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
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.08, 0.08, 0.08, 1.0].into()), Some(1f32.into())],
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
        vulkan: &Vulkan,
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
            .then_execute(vulkan.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                vulkan.queue.clone(),
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
                rcx.previous_frame_end = Some(sync::now(vulkan.device.clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                rcx.previous_frame_end = Some(sync::now(vulkan.device.clone()).boxed());
            }
        }
    }

    pub fn update_time(&mut self) {
        let time_info = &mut self.rcx.as_mut().unwrap().time_info;

        time_info.dt = time_info.time.elapsed().as_secs_f32();
        time_info.time = Instant::now();
        time_info.dt_sum += time_info.dt;
        time_info.frame_count += 1.0;

        // calculate the fps every second
        if time_info.dt_sum > 1.0 {
            time_info.avg_fps = time_info.frame_count / time_info.dt_sum;
            time_info.dt_sum = 0.0;
            time_info.frame_count = 0.0;
        }
    }

    pub fn update_title(&mut self, title: &str) {
        let rcx = self.rcx.as_ref().unwrap();

        rcx.window.set_title(title);
    }
}
