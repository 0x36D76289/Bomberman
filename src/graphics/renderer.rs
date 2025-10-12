use crate::{
    app_state::AppState,
    game::resources::Resources,
    graphics::{GameVertex, GuiVertex, TextRenderer, TimeInfo, Vulkan},
    settings::settings::Settings,
};
use std::{sync::Arc, time::Instant};
use vulkano::{
    Validated, VulkanError,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer},
    descriptor_set::layout::DescriptorBindingFlags,
    format::Format,
    image::{
        Image, ImageCreateInfo, ImageType, ImageUsage,
        sampler::{BorderColor, Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
        view::ImageView,
    },
    memory::allocator::AllocationCreateInfo,
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            subpass::{PipelineRenderingCreateInfo, PipelineSubpassType},
            vertex_input::{Vertex, VertexDefinition, VertexInputState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    shader::EntryPoint,
    swapchain::{
        ColorSpace, PresentMode, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
        SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture},
};
use winit::{
    event_loop::ActiveEventLoop,
    window::{Fullscreen, Window},
};

pub struct Renderer {
    pub rcx: Option<RenderContext>,
    pub game_pipeline: Option<Arc<GraphicsPipeline>>,
    pub gui_pipeline: Option<Arc<GraphicsPipeline>>,
    pub post_process_pipeline: Option<Arc<GraphicsPipeline>>,
    pub command_buffer: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    pub text_renderer: TextRenderer,
    pub sampler: Arc<Sampler>,
}

pub struct RenderContext {
    pub window: Arc<Window>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<ImageView>>,
    pub color_image: Arc<ImageView>,
    pub depth_image: Arc<ImageView>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub time_info: TimeInfo,
    pub game_resolution: Resolution,
    pub fullscreen: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    Full,
    Custom(u32, u32),
}

impl Resolution {
    pub fn resolution(&self, window_size: [u32; 2]) -> [u32; 2] {
        match self {
            Resolution::Full => window_size,
            Resolution::Custom(width, height) => [*width, *height],
        }
    }
}

#[derive(Debug, Clone)]
/// Represents how a state is rendered
pub struct StateRenderInfo {
    pub top_left_coord: [f32; 2],
    pub bottom_right_coord: [f32; 2],
    pub drawn_first: bool,
}

impl Default for StateRenderInfo {
    fn default() -> Self {
        Self {
            top_left_coord: [-1.0, -1.0],
            bottom_right_coord: [1.0, 1.0],
            drawn_first: false,
        }
    }
}

impl Renderer {
    pub fn new(vulkan: &Vulkan) -> Self {
        let text_renderer = TextRenderer::new();

        let sampler = Sampler::new(
            vulkan.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                address_mode: [SamplerAddressMode::ClampToBorder; 3],
                border_color: BorderColor::FloatOpaqueWhite,
                ..Default::default()
            },
        )
        .unwrap();

        Self {
            rcx: None,
            game_pipeline: None,
            gui_pipeline: None,
            post_process_pipeline: None,
            command_buffer: None,
            text_renderer,
            sampler,
        }
    }

    pub fn init_render_context(
        &mut self,
        event_loop: &ActiveEventLoop,
        vulkan: &Vulkan,
        settings: &Settings,
    ) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Bomberman!"))
                .unwrap(),
        );

        set_fullscreen(settings.fullscreen, &window);

        let surface = Surface::from_window(vulkan.instance.clone(), window.clone())
            .expect("Could not create surface");

        let window_size: [u32; 2] = window.inner_size().into();
        let game_resolution = Resolution::Custom(settings.resolution.0, settings.resolution.1);

        // Create the swapchain which holds a queue of images that are waiting to be presented on the screen
        let (swapchain, images) = {
            let surface_capabilities = vulkan
                .device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let image_formats = vulkan
                .device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap();

            let (image_format, _) =
                if image_formats.contains(&(Format::B8G8R8A8_UNORM, ColorSpace::SrgbNonLinear)) {
                    (Format::B8G8R8A8_UNORM, ColorSpace::SrgbNonLinear)
                } else {
                    println!(
                        "Warning: the device doesnt support B8G8R8A8_UNORM, the colors might be off"
                    );
                    image_formats[0]
                };

            Swapchain::new(
                vulkan.device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window_size,
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    present_mode: PresentMode::Fifo,
                    ..Default::default()
                },
            )
            .unwrap()
        };

        let (color_image, depth_image) = create_images(
            vulkan,
            game_resolution.resolution(window_size),
            images[0].format(),
            Format::D16_UNORM,
        );

        let images = images
            .iter()
            .map(|image| ImageView::new_default(image.clone()).unwrap())
            .collect();

        let recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(vulkan.device.clone()).boxed());

        let time_info = TimeInfo {
            time: Instant::now(),
            dt: 0.0,
            frame_count: 0.0,
            avg_fps: 0.0,
            dt_sum: 0.0,
        };

        self.rcx = Some(RenderContext {
            window,
            swapchain,
            images,
            color_image,
            depth_image,
            recreate_swapchain,
            previous_frame_end,
            time_info,
            game_resolution,
            fullscreen: settings.fullscreen,
        })
    }

    pub fn create_pipelines(&mut self, vulkan: &Vulkan) {
        self.game_pipeline = {
            let vertex_shader = game_vs::load(vulkan.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let fragment_shader = game_fs::load(vulkan.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let vertex_input_state = GameVertex::per_vertex().definition(&vertex_shader).unwrap();
            Some(self.create_pipeline(
                vulkan,
                vertex_shader,
                Some(fragment_shader),
                vertex_input_state,
                true,
                true,
                Some(2),
            ))
        };
        self.gui_pipeline = {
            let vertex_shader = gui_vs::load(vulkan.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let fragment_shader = gui_fs::load(vulkan.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let vertex_input_state = GuiVertex::per_vertex().definition(&vertex_shader).unwrap();
            Some(self.create_pipeline(
                vulkan,
                vertex_shader,
                Some(fragment_shader),
                vertex_input_state,
                true,
                false,
                Some(1),
            ))
        };
        self.post_process_pipeline = {
            let vertex_shader = postprocess_vs::load(vulkan.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let fragment_shader = postprocess_fs::load(vulkan.device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let vertex_input_state = VertexInputState::new();
            Some(self.create_pipeline(
                vulkan,
                vertex_shader,
                Some(fragment_shader),
                vertex_input_state,
                true,
                false,
                None,
            ))
        };
    }

    fn create_pipeline(
        &self,
        vulkan: &Vulkan,
        vertex_shader: EntryPoint,
        fragment_shader: Option<EntryPoint>,
        vertex_input_state: VertexInputState,
        has_color_attachment: bool,
        has_depth_attachment: bool,
        variable_descriptor_count: Option<u32>,
    ) -> Arc<GraphicsPipeline> {
        let stages = match fragment_shader {
            Some(fragment_shader) => vec![
                PipelineShaderStageCreateInfo::new(vertex_shader.clone()),
                PipelineShaderStageCreateInfo::new(fragment_shader.clone()),
            ],
            None => vec![PipelineShaderStageCreateInfo::new(vertex_shader.clone())],
        };

        let layout = {
            let mut layout_create_info =
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages);

            if let Some(index) = variable_descriptor_count {
                let binding = layout_create_info.set_layouts[0]
                    .bindings
                    .get_mut(&index)
                    .unwrap();
                binding.binding_flags |= DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT;
                binding.descriptor_count = 100;
            }

            PipelineLayout::new(
                vulkan.device.clone(),
                layout_create_info
                    .into_pipeline_layout_create_info(vulkan.device.clone())
                    .unwrap(),
            )
            .unwrap()
        };

        let mut pipeline_rendering_info = PipelineRenderingCreateInfo::default();
        let mut depth_stencil_state = None;
        let mut color_blend_state = None;

        if has_color_attachment {
            let format = self.rcx().swapchain.image_format();
            pipeline_rendering_info.color_attachment_formats = vec![Some(format)];
            color_blend_state = Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    ..Default::default()
                },
            ));
        }
        if has_depth_attachment {
            pipeline_rendering_info.depth_attachment_format = Some(Format::D16_UNORM);
            depth_stencil_state = Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::Less,
                }),
                ..Default::default()
            });
        }

        GraphicsPipeline::new(
            vulkan.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                viewport_state: Some(Default::default()),
                color_blend_state,
                input_assembly_state: Some(InputAssemblyState::default()),
                rasterization_state: Some(RasterizationState::default()),
                depth_stencil_state,
                multisample_state: Some(MultisampleState::default()),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(PipelineSubpassType::BeginRendering(pipeline_rendering_info)),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    }

    pub fn rcx(&self) -> &RenderContext {
        self.rcx.as_ref().unwrap()
    }

    pub fn is_initialized(&self) -> bool {
        self.rcx.is_some()
            && self.game_pipeline.is_some()
            && self.gui_pipeline.is_some()
            && self.post_process_pipeline.is_some()
    }

    pub fn get_delta_time(&self) -> f32 {
        self.rcx.as_ref().unwrap().time_info.dt
    }

    #[allow(unused)]
    pub fn window_size(&self) -> [u32; 2] {
        self.rcx().swapchain.image_extent()
    }

    pub fn recreate_swapchain(&mut self, b: bool) {
        self.rcx.as_mut().unwrap().recreate_swapchain = b;
    }

    pub fn request_redraw(&self) {
        self.rcx.as_ref().unwrap().window.request_redraw();
    }

    pub fn render_state(
        &mut self,
        vulkan: &Vulkan,
        resources: &Resources,
        state: &AppState,
        image_index: u32,
        is_first: bool,
    ) {
        match (&state.game, &state.ui) {
            (Some(game_state), Some(ui_state)) => {
                if ui_state.render_info.drawn_first && !game_state.render_info.drawn_first {
                    self.render_ui(vulkan, resources, ui_state, image_index, is_first);
                    self.render_game(vulkan, resources, game_state, image_index, false)
                } else {
                    self.render_game(vulkan, resources, game_state, image_index, is_first);
                    self.render_ui(vulkan, resources, ui_state, image_index, false);
                }
            }
            (Some(game_state), None) => {
                self.render_game(vulkan, resources, game_state, image_index, is_first)
            }
            (None, Some(ui_state)) => {
                self.render_ui(vulkan, resources, ui_state, image_index, is_first)
            }
            (None, None) => (),
        }
    }

    pub fn render_states(&mut self, vulkan: &Vulkan, states: &[AppState], resources: &Resources) {
        let rcx = self.rcx.as_mut().unwrap();

        let window_size = rcx.window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        let states_to_render = states_to_render(states);
        if states_to_render.is_empty() {
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
            rcx.images = new_images
                .iter()
                .map(|image| ImageView::new_default(image.clone()).unwrap())
                .collect();
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

        self.command_buffer = Some(
            AutoCommandBufferBuilder::primary(
                vulkan.command_buffer_allocator.clone(),
                vulkan.queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap(),
        );

        let mut is_first = true;
        for state in states_to_render {
            self.render_state(vulkan, resources, state, image_index, is_first);
            if is_first {
                is_first = false
            }
        }

        self.execute_command_buffer(vulkan, acquire_future, image_index);
    }

    fn execute_command_buffer(
        &mut self,
        vulkan: &Vulkan,
        acquire_future: SwapchainAcquireFuture,
        image_index: u32,
    ) {
        let (command_buffer, rcx) = match (self.command_buffer.take(), self.rcx.as_mut()) {
            (Some(command_buffer), Some(rcx)) => (command_buffer, rcx),
            (None, _) => panic!("Tried to execute the command buffer but its state is None"),
            (_, None) => panic!("Render context is not initialized"),
        };

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
    }

    pub fn update_settings(&mut self, vulkan: &Vulkan, settings: &Settings) {
        let rcx = self.rcx.as_mut().unwrap();

        let settings_resolution = Resolution::Custom(settings.resolution.0, settings.resolution.1);
        if rcx.game_resolution != settings_resolution {
            rcx.game_resolution = settings_resolution;
            (rcx.color_image, rcx.depth_image) = create_images(
                vulkan,
                rcx.game_resolution
                    .resolution(rcx.window.inner_size().into()),
                rcx.images[0].format(),
                Format::D16_UNORM,
            );
        }

        if rcx.fullscreen != settings.fullscreen {
            rcx.fullscreen = settings.fullscreen;
            set_fullscreen(rcx.fullscreen, &rcx.window);
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

fn create_images(
    vulkan: &Vulkan,
    resolution: [u32; 2],
    color_format: Format,
    depth_format: Format,
) -> (Arc<ImageView>, Arc<ImageView>) {
    let color_image = ImageView::new_default(
        Image::new(
            vulkan.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: color_format,
                extent: [resolution[0], resolution[1], 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let depth_image = ImageView::new_default(
        Image::new(
            vulkan.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: depth_format,
                extent: [resolution[0], resolution[1], 1],
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    (color_image, depth_image)
}

fn states_to_render(states: &[AppState]) -> Vec<&AppState> {
    let transparent_count = states
        .iter()
        .rev()
        .take_while(|s| s.is_transparent())
        .count();
    let states_to_skip = if states.len() > 0 {
        (states.len() - 1).saturating_sub(transparent_count)
    } else {
        0
    };
    states.iter().skip(states_to_skip).collect()
}

fn set_fullscreen(fullscreen: bool, window: &Window) {
    if let Some(current_monitor) = window.current_monitor() {
        if fullscreen {
            #[cfg(target_os = "macos")]
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(current_monitor))));

            #[cfg(not(target_os = "macos"))]
            if let Some(video_mode) = current_monitor.video_modes().next() {
                window.set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
            } else {
                window.set_fullscreen(Some(Fullscreen::Borderless(Some(current_monitor))));
            }
        } else {
            window.set_fullscreen(None);
        }
    }
}

pub mod game_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/game.vert"
    }
}

pub mod game_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/game.frag"
    }
}

pub mod gui_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/gui.vert"
    }
}

pub mod gui_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/gui.frag"
    }
}

pub mod postprocess_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/post_process.vert"
    }
}

pub mod postprocess_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/post_process.frag"
    }
}
