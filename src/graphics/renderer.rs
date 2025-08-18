use crate::{app_state::AppState, graphics::{GameRenderSystem, TimeInfo, UiRenderSystem, Vulkan}};
use std::{sync::Arc, time::Instant};
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
    }, format::Format, image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage}, memory::allocator::AllocationCreateInfo, pipeline::graphics::viewport::Viewport, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, swapchain::{
        acquire_next_image, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo
    }, sync::{self, GpuFuture}, Validated, VulkanError
};
use winit::{event_loop::ActiveEventLoop, window::Window};

pub struct Renderer {
    rcx: Option<RenderContext>,
    image_index: u32,
    acquire_future: Option<SwapchainAcquireFuture>,
    primary_command_buffer: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    game_render_system: Option<GameRenderSystem>,
    ui_render_system: Option<UiRenderSystem>
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
            primary_command_buffer: None,
            game_render_system: None,
            ui_render_system: None
        }
    }

    pub fn init_render_context(&mut self, event_loop: &ActiveEventLoop, vulkan: &Vulkan) {
        self.rcx = Some(RenderContext::init(event_loop, &vulkan).unwrap())
    }

    pub fn init_game_render_system(&mut self, vulkan: &Vulkan) {
        let mut game_render_system = GameRenderSystem::default();
        game_render_system.create_pipeline(vulkan, self.rcx().render_pass.clone());
        self.game_render_system = Some(game_render_system);
    }

    pub fn init_ui_render_system(&mut self, _vulkan: &Vulkan) {
        let ui_render_system = UiRenderSystem{};
        self.ui_render_system = Some(ui_render_system);
    }

    pub fn rcx(&self) -> &RenderContext {
        self.rcx.as_ref().unwrap()
    }

    pub fn game_render_system(&self) -> &GameRenderSystem {
        self.game_render_system.as_ref().unwrap()
    }

    pub fn ui_render_system(&self) -> &UiRenderSystem {
        self.ui_render_system.as_ref().unwrap()
    }

    pub fn get_delta_time(&self) -> f32 {
        self.rcx.as_ref().unwrap().time_info.dt
    }

    pub fn window_size(&self) -> [u32; 2] {
        self.rcx().swapchain.image_extent()
    }

    pub fn recreate_swapchain(&mut self, b: bool) {
        self.rcx.as_mut().unwrap().recreate_swapchain = b;
    }

    pub fn request_redraw(&self) {
        self.rcx.as_ref().unwrap().window.request_redraw();
    }

    fn begin_frame(&mut self, vulkan: &Vulkan) {
        let rcx = self.rcx.as_mut().unwrap();

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
                    return;
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
                SubpassBeginInfo { 
                    contents: SubpassContents::SecondaryCommandBuffers,
                    ..Default::default()
                }
            )
            .unwrap();

        self.primary_command_buffer = Some(builder);
    }

    fn end_frame(
        &mut self,
        vulkan: &Vulkan,
    ) {
        let rcx = self.rcx.as_mut().unwrap();

        let mut command_buffer = match self.primary_command_buffer.take() {
            Some(cb) => cb,
            None => return
        };

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

    fn render_frame(&mut self, vulkan: &Vulkan, game_state_back: Option<&AppState>, game_state_front: Option<&AppState>) {
        let mut primary_cb = match self.primary_command_buffer.take() {
            Some(cb) => cb,
            None => return
        };

        match (game_state_back, game_state_front) {
            (None, None) => (),
            (Some(state), None) | (None, Some(state)) => {
                
                primary_cb.next_subpass(
                    Default::default(),
                    SubpassBeginInfo {
                        contents: SubpassContents::SecondaryCommandBuffers,
                        ..Default::default()
                    }
                )
                .unwrap();
                let secondary_cb = state.render(self, vulkan);
                primary_cb.execute_commands(secondary_cb).unwrap();
            }
            (Some(state1), Some(state2)) => {
                let secondary_cb = state1.render(self, vulkan);
                primary_cb.execute_commands(secondary_cb).unwrap();
                primary_cb.next_subpass(
                    Default::default(),
                    SubpassBeginInfo {
                        contents: SubpassContents::SecondaryCommandBuffers,
                        ..Default::default()
                    }
                )
                .unwrap();
                let secondary_cb = state2.render(self, vulkan);
                primary_cb.execute_commands(secondary_cb).unwrap();
            }
        }

        self.primary_command_buffer = Some(primary_cb);
    }

    pub fn render(&mut self, vulkan: &Vulkan, states: &Vec<AppState>) {
        let state = states.last().unwrap();

        self.begin_frame(vulkan);
        self.render_frame(vulkan, Some(state), None);
        self.end_frame(vulkan);
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
