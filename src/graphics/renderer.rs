use crate::{
    app_state::AppState,
    game::resources::Resources,
    graphics::{AnimationVertex, GameVertex, GuiVertex, TimeInfo, UiVertex, Vulkan},
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
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
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
            vertex_input::{Vertex, VertexDefinition},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    shader::ShaderModule,
    swapchain::{
        ColorSpace, PresentMode, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
        SwapchainPresentInfo, acquire_next_image,
    },
    sync::{self, GpuFuture},
};
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Fullscreen, Window},
};

fn window_icon() -> Option<winit::window::Icon> {
    let image =
        image::load_from_memory(&crate::settings::assets::read_asset("textures/icon.png").ok()?)
            .ok()?
            .into_rgba8();
    let (width, height) = image.dimensions();
    winit::window::Icon::from_rgba(image.into_raw(), width, height).ok()
}

fn game_sampler(vulkan: &Vulkan, filtering: bool) -> Arc<Sampler> {
    let filter = if filtering {
        Filter::Linear
    } else {
        Filter::Nearest
    };
    Sampler::new(
        vulkan.device.clone(),
        SamplerCreateInfo {
            mag_filter: filter,
            min_filter: filter,
            address_mode: [SamplerAddressMode::Repeat; 3],
            ..Default::default()
        },
    )
    .unwrap()
}

fn present_mode(framerate_limit: i32, supported: &[PresentMode]) -> PresentMode {
    if framerate_limit == 0 {
        return PresentMode::Fifo;
    }
    [PresentMode::Mailbox, PresentMode::Immediate]
        .into_iter()
        .find(|mode| supported.contains(mode))
        .unwrap_or(PresentMode::Fifo)
}

pub struct Renderer {
    pub rcx: Option<RenderContext>,
    pub game_pipeline: Option<Arc<GraphicsPipeline>>,
    pub animation_pipeline: Option<Arc<GraphicsPipeline>>,
    pub gui_pipeline: Option<Arc<GraphicsPipeline>>,
    pub post_process_pipeline: Option<Arc<GraphicsPipeline>>,
    pub command_buffer: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    pub ui_sampler: Arc<Sampler>,
    pub game_sampler: Arc<Sampler>,
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
    pub window_resolution: (u32, u32),
    pub present_modes: Vec<PresentMode>,
    pub present_mode: PresentMode,
}

const DEPTH_FORMAT: Format = Format::D16_UNORM;

#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    Full,
    Pixel(u32),
}

impl Resolution {
    pub fn from_settings(settings: &Settings) -> Self {
        if settings.pixel_art {
            Resolution::Pixel(270)
        } else {
            Resolution::Full
        }
    }

    pub fn resolution(&self, window_size: [u32; 2]) -> [u32; 2] {
        match self {
            Resolution::Full => window_size,
            Resolution::Pixel(height) => window_size.map(|side| {
                side.div_ceil((window_size[1] as f32 / *height as f32).round().max(1.0) as u32)
                    .max(1)
            }),
        }
    }
}

#[derive(Debug, Clone)]
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
        Self {
            rcx: None,
            game_pipeline: None,
            animation_pipeline: None,
            gui_pipeline: None,
            post_process_pipeline: None,
            command_buffer: None,
            ui_sampler: Sampler::new(
                vulkan.device.clone(),
                SamplerCreateInfo {
                    mag_filter: Filter::Nearest,
                    min_filter: Filter::Linear,
                    ..Default::default()
                },
            )
            .unwrap(),
            game_sampler: game_sampler(vulkan, false),
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
                .create_window(
                    Window::default_attributes()
                        .with_title("Bomberman")
                        .with_window_icon(window_icon())
                        .with_inner_size(PhysicalSize::new(
                            settings.resolution.0,
                            settings.resolution.1,
                        )),
                )
                .expect("could not create the window"),
        );

        set_fullscreen(settings.fullscreen, &window);

        let surface = Surface::from_window(vulkan.instance.clone(), window.clone())
            .expect("Could not create surface");

        let window_size: [u32; 2] = window.inner_size().into();
        let game_resolution = Resolution::from_settings(settings);
        let present_modes: Vec<PresentMode> = vulkan
            .device
            .physical_device()
            .surface_present_modes(&surface, Default::default())
            .unwrap_or_default();
        let present_mode = present_mode(settings.framerate_limit, &present_modes);

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
                    image_formats[0]
                };

            Swapchain::new(
                vulkan.device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window_size,
                    image_usage: ImageUsage::COLOR_ATTACHMENT
                        | (surface_capabilities.supported_usage_flags & ImageUsage::TRANSFER_SRC),
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    present_mode,
                    ..Default::default()
                },
            )
            .unwrap()
        };

        let (color_image, depth_image) = create_images(
            vulkan,
            game_resolution.resolution(window_size),
            images[0].format(),
        );

        self.rcx = Some(RenderContext {
            window,
            swapchain,
            images: images
                .iter()
                .map(|image| ImageView::new_default(image.clone()).unwrap())
                .collect(),
            color_image,
            depth_image,
            recreate_swapchain: false,
            previous_frame_end: Some(sync::now(vulkan.device.clone()).boxed()),
            time_info: TimeInfo {
                time: Instant::now(),
                dt: 0.0,
            },
            game_resolution,
            fullscreen: settings.fullscreen,
            window_resolution: settings.resolution,
            present_modes,
            present_mode,
        })
    }

    pub fn create_pipelines(&mut self, vulkan: &Vulkan) {
        let device = &vulkan.device;
        self.game_pipeline = Some(self.create_pipeline::<GameVertex>(
            vulkan,
            game_vs::load(device.clone()),
            game_fs::load(device.clone()),
            true,
            Some(2),
        ));
        self.animation_pipeline = Some(self.create_pipeline::<AnimationVertex>(
            vulkan,
            animation_vs::load(device.clone()),
            animation_fs::load(device.clone()),
            true,
            Some(2),
        ));
        self.gui_pipeline = Some(self.create_pipeline::<UiVertex>(
            vulkan,
            gui_vs::load(device.clone()),
            gui_fs::load(device.clone()),
            false,
            Some(1),
        ));
        self.post_process_pipeline = Some(self.create_pipeline::<GuiVertex>(
            vulkan,
            postprocess_vs::load(device.clone()),
            postprocess_fs::load(device.clone()),
            false,
            None,
        ));
    }

    fn create_pipeline<V: Vertex>(
        &self,
        vulkan: &Vulkan,
        vertex_shader: Result<Arc<ShaderModule>, Validated<VulkanError>>,
        fragment_shader: Result<Arc<ShaderModule>, Validated<VulkanError>>,
        has_depth_attachment: bool,
        variable_descriptor_count: Option<u32>,
    ) -> Arc<GraphicsPipeline> {
        let vertex_shader = vertex_shader.unwrap().entry_point("main").unwrap();
        let vertex_input_state = V::per_vertex().definition(&vertex_shader).unwrap();
        let stages = [
            PipelineShaderStageCreateInfo::new(vertex_shader),
            PipelineShaderStageCreateInfo::new(
                fragment_shader.unwrap().entry_point("main").unwrap(),
            ),
        ];

        let layout = {
            let mut layout_create_info =
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages);

            if let Some(index) = variable_descriptor_count {
                let binding = layout_create_info.set_layouts[0]
                    .bindings
                    .get_mut(&index)
                    .unwrap();
                binding.binding_flags |= DescriptorBindingFlags::VARIABLE_DESCRIPTOR_COUNT;
                binding.descriptor_count = 256;
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

        pipeline_rendering_info.color_attachment_formats =
            vec![Some(self.rcx().swapchain.image_format())];
        if has_depth_attachment {
            pipeline_rendering_info.depth_attachment_format = Some(DEPTH_FORMAT);
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
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    1,
                    ColorBlendAttachmentState {
                        blend: Some(AttachmentBlend::alpha()),
                        ..Default::default()
                    },
                )),
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
            && self.animation_pipeline.is_some()
            && self.gui_pipeline.is_some()
            && self.post_process_pipeline.is_some()
    }

    pub fn get_delta_time(&self) -> f32 {
        self.rcx().time_info.dt
    }

    pub fn window_size(&self) -> [u32; 2] {
        self.rcx().swapchain.image_extent()
    }

    pub fn recreate_swapchain(&mut self, b: bool) {
        self.rcx.as_mut().unwrap().recreate_swapchain = b;
    }

    pub fn request_redraw(&self) {
        self.rcx().window.request_redraw();
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

    pub fn render_states(
        &mut self,
        vulkan: &Vulkan,
        states: &[AppState],
        resources: &Resources,
        overlay: Option<&crate::ui::UiState>,
    ) {
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
                    present_mode: rcx.present_mode,
                    ..rcx.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            (rcx.color_image, rcx.depth_image) = create_images(
                vulkan,
                rcx.game_resolution.resolution(window_size.into()),
                new_images[0].format(),
            );

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

        for (i, state) in states_to_render.into_iter().enumerate() {
            self.render_state(vulkan, resources, state, image_index, i == 0);
        }
        if let Some(overlay) = overlay {
            self.render_ui(vulkan, resources, overlay, image_index, false);
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

        match rcx
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(vulkan.queue.clone(), command_buffer.build().unwrap())
            .unwrap()
            .then_signal_semaphore()
            .then_swapchain_present(
                vulkan.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(rcx.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush()
            .map_err(Validated::unwrap)
        {
            Ok(future) => {
                rcx.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                rcx.recreate_swapchain = true;
                rcx.previous_frame_end = Some(sync::now(vulkan.device.clone()).boxed());
            }
            Err(_) => {
                rcx.previous_frame_end = Some(sync::now(vulkan.device.clone()).boxed());
            }
        }
    }

    pub fn update_settings(&mut self, vulkan: &Vulkan, settings: &Settings) {
        let rcx = self.rcx.as_mut().unwrap();

        let resized = rcx.window_resolution != settings.resolution;
        let toggled = rcx.fullscreen != settings.fullscreen;
        rcx.window_resolution = settings.resolution;
        rcx.fullscreen = settings.fullscreen;
        if toggled {
            set_fullscreen(rcx.fullscreen, &rcx.window);
        }
        if (resized || toggled) && !rcx.fullscreen {
            let _ = rcx.window.request_inner_size(PhysicalSize::new(
                settings.resolution.0,
                settings.resolution.1,
            ));
        }

        let wanted = present_mode(settings.framerate_limit, &rcx.present_modes);
        if wanted != rcx.present_mode {
            rcx.present_mode = wanted;
            rcx.recreate_swapchain = true;
        }

        let resolution = Resolution::from_settings(settings);
        if rcx.game_resolution != resolution {
            rcx.game_resolution = resolution;
            rcx.recreate_swapchain = true;
        }

        let smooth = settings.filtering && !settings.pixel_art;
        if smooth != (self.game_sampler.mag_filter() == Filter::Linear) {
            self.game_sampler = game_sampler(vulkan, smooth);
        }
    }

    pub fn update_time(&mut self) {
        let time_info = &mut self.rcx.as_mut().unwrap().time_info;

        time_info.dt = time_info.time.elapsed().as_secs_f32();
        time_info.time = Instant::now();
    }
}

fn attachment(
    vulkan: &Vulkan,
    resolution: [u32; 2],
    format: Format,
    usage: ImageUsage,
) -> Arc<ImageView> {
    ImageView::new_default(
        Image::new(
            vulkan.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format,
                extent: [resolution[0], resolution[1], 1],
                usage,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn create_images(
    vulkan: &Vulkan,
    resolution: [u32; 2],
    color_format: Format,
) -> (Arc<ImageView>, Arc<ImageView>) {
    (
        attachment(
            vulkan,
            resolution,
            color_format,
            ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
        ),
        attachment(
            vulkan,
            resolution,
            DEPTH_FORMAT,
            ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
        ),
    )
}

fn states_to_render(states: &[AppState]) -> Vec<&AppState> {
    let Some(top) = states.last() else {
        return Vec::new();
    };
    top.is_transparent()
        .then(|| states.iter().rev().find(|s| !s.is_transparent()))
        .flatten()
        .into_iter()
        .chain(std::iter::once(top))
        .collect()
}

fn set_fullscreen(fullscreen: bool, window: &Window) {
    if let Some(current_monitor) = window.current_monitor() {
        if fullscreen {
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(current_monitor))));
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

pub mod animation_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/game_animation.vert"
    }
}

pub mod animation_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/game_animation.frag"
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
