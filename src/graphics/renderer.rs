use crate::{
    app_state::AppState,
    game::resources::Resources,
    graphics::{GameVertex, GuiVertex, TextRenderer, TimeInfo, Vulkan},
};
use std::{sync::Arc, time::Instant};
use vulkano::{
    Validated, VulkanError,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderingAttachmentInfo, RenderingInfo, SubpassContents,
    },
    descriptor_set::{DescriptorSet, WriteDescriptorSet, layout::DescriptorBindingFlags},
    format::{ClearValue, Format},
    image::{
        Image, ImageCreateInfo, ImageType, ImageUsage,
        sampler::{BorderColor, Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
        view::ImageView,
    },
    memory::allocator::AllocationCreateInfo,
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            subpass::{PipelineRenderingCreateInfo, PipelineSubpassType},
            vertex_input::{Vertex, VertexDefinition, VertexInputState},
            viewport::Viewport,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
    swapchain::{
        ColorSpace, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
        acquire_next_image,
    },
    sync::{self, GpuFuture},
};
use winit::{event_loop::ActiveEventLoop, window::Window};

pub const RENDER_RES_RATIO: [u32; 2] = [1, 1];

pub struct Renderer {
    rcx: Option<RenderContext>,
    pub game_pipeline: Option<Arc<GraphicsPipeline>>,
    pub gui_pipeline: Option<Arc<GraphicsPipeline>>,
    pub pixelate_pipeline: Option<Arc<GraphicsPipeline>>,
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
            pixelate_pipeline: None,
            text_renderer,
            sampler,
        }
    }

    pub fn init_render_context(&mut self, event_loop: &ActiveEventLoop, vulkan: &Vulkan) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Bomberman!"))
                .unwrap(),
        );

        let surface = Surface::from_window(vulkan.instance.clone(), window.clone())
            .expect("Could not create surface");

        let window_size: [u32; 2] = window.inner_size().into();
        let game_resolution = [
            window_size[0] / RENDER_RES_RATIO[0],
            window_size[1] / RENDER_RES_RATIO[1],
        ];

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

        let color_image = ImageView::new_default(
            Image::new(
                vulkan.memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: images[0].format(),
                    extent: [game_resolution[0], game_resolution[1], 1],
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
                    format: Format::D32_SFLOAT,
                    extent: [game_resolution[0], game_resolution[1], 1],
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();
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
        })
    }

    pub fn create_game_pipeline(&mut self, vulkan: &Vulkan) {
        let vertex_shader = game_vs::load(vulkan.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fragment_shader = game_fs::load(vulkan.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = GameVertex::per_vertex().definition(&vertex_shader).unwrap();
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

        let format = self.rcx().swapchain.image_format();

        let pipeline_rendering_info = PipelineRenderingCreateInfo {
            color_attachment_formats: vec![Some(format)],
            depth_attachment_format: Some(Format::D32_SFLOAT),
            ..Default::default()
        };

        self.game_pipeline = Some(
            GraphicsPipeline::new(
                vulkan.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    viewport_state: Some(Default::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        1,
                        ColorBlendAttachmentState::default(),
                    )),
                    input_assembly_state: Some(InputAssemblyState::default()),
                    rasterization_state: Some(RasterizationState::default()),
                    depth_stencil_state: Some(DepthStencilState {
                        depth: Some(DepthState::simple()),
                        ..Default::default()
                    }),
                    multisample_state: Some(MultisampleState::default()),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(PipelineSubpassType::BeginRendering(pipeline_rendering_info)),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap(),
        );
    }

    pub fn create_gui_pipeline(&mut self, vulkan: &Vulkan) {
        let vertex_shader = gui_vs::load(vulkan.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fragment_shader = gui_fs::load(vulkan.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = GuiVertex::per_vertex().definition(&vertex_shader).unwrap();
        let stages = [
            PipelineShaderStageCreateInfo::new(vertex_shader.clone()),
            PipelineShaderStageCreateInfo::new(fragment_shader.clone()),
        ];
        let layout = {
            let mut layout_create_info =
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages);

            let binding = layout_create_info.set_layouts[0]
                .bindings
                .get_mut(&1)
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

        let format = self.rcx().swapchain.image_format();

        let pipeline_rendering_info = PipelineRenderingCreateInfo {
            color_attachment_formats: vec![Some(format)],
            ..Default::default()
        };

        self.gui_pipeline = Some(
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
                    multisample_state: Some(MultisampleState::default()),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(PipelineSubpassType::BeginRendering(pipeline_rendering_info)),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap(),
        );
    }

    pub fn create_postprocess_pipeline(&mut self, vulkan: &Vulkan) {
        let vertex_shader = postprocess_vs::load(vulkan.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fragment_shader = postprocess_fs::load(vulkan.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = VertexInputState::new();
        let stages = [
            PipelineShaderStageCreateInfo::new(vertex_shader.clone()),
            PipelineShaderStageCreateInfo::new(fragment_shader.clone()),
        ];
        let layout = {
            let layout_create_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages);

            PipelineLayout::new(
                vulkan.device.clone(),
                layout_create_info
                    .into_pipeline_layout_create_info(vulkan.device.clone())
                    .unwrap(),
            )
            .unwrap()
        };

        let format = self.rcx().swapchain.image_format();

        let pipeline_rendering_info = PipelineRenderingCreateInfo {
            color_attachment_formats: vec![Some(format)],
            ..Default::default()
        };

        self.pixelate_pipeline = Some(
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
                    multisample_state: Some(MultisampleState::default()),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(PipelineSubpassType::BeginRendering(pipeline_rendering_info)),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap(),
        );
    }

    pub fn rcx(&self) -> &RenderContext {
        self.rcx.as_ref().unwrap()
    }

    pub fn is_initialized(&self) -> bool {
        self.rcx.is_some()
            && self.game_pipeline.is_some()
            && self.gui_pipeline.is_some()
            && self.pixelate_pipeline.is_some()
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
        &self,
        vulkan: &Vulkan,
        primary_cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        state: &AppState,
        resources: &Resources,
        image_index: u32,
        is_first: bool,
    ) {
        let rcx = self.rcx.as_ref().unwrap();

        let pass_info = match state {
            AppState::Game(_) => {
                let mut color_attachment =
                    RenderingAttachmentInfo::image_view(rcx.color_image.clone());
                color_attachment.store_op = AttachmentStoreOp::Store;
                color_attachment.load_op = AttachmentLoadOp::Load;
                if is_first {
                    color_attachment.load_op = AttachmentLoadOp::Clear;
                    color_attachment.clear_value = Some(ClearValue::Float([0.0, 0.0, 0.0, 0.0]));
                }
                let mut depth_attachment =
                    RenderingAttachmentInfo::image_view(rcx.depth_image.clone());
                depth_attachment.load_op = AttachmentLoadOp::Clear;
                depth_attachment.clear_value = Some(ClearValue::DepthStencil((1.0, 0)));
                RenderingInfo {
                    color_attachments: vec![Some(color_attachment)],
                    depth_attachment: Some(depth_attachment),
                    layer_count: 1,
                    contents: SubpassContents::SecondaryCommandBuffers,
                    ..Default::default()
                }
            }
            AppState::Ui(_) => {
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
                    contents: SubpassContents::SecondaryCommandBuffers,
                    ..Default::default()
                }
            }
        };

        primary_cb.begin_rendering(pass_info).unwrap();

        let secondary_cb = state.render(self, vulkan, resources);

        primary_cb.execute_commands(secondary_cb).unwrap();

        primary_cb.end_rendering().unwrap();

        if let AppState::Game(_) = state {
            self.render_upscale_quad(vulkan, primary_cb, image_index);
        }
    }

    pub fn render_upscale_quad(
        &self,
        vulkan: &Vulkan,
        primary_cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        image_index: u32,
    ) {
        let rcx = self.rcx.as_ref().unwrap();
        let pipeline = self.pixelate_pipeline.as_ref().unwrap().clone();

        let mut color_attachment =
            RenderingAttachmentInfo::image_view(rcx.images[image_index as usize].clone());
        color_attachment.load_op = AttachmentLoadOp::Clear;
        color_attachment.clear_value = Some(ClearValue::Float([0.0, 0.0, 0.0, 0.0]));
        color_attachment.store_op = AttachmentStoreOp::Store;

        let pass_info = RenderingInfo {
            color_attachments: vec![Some(color_attachment)],
            layer_count: 1,
            contents: SubpassContents::Inline,
            ..Default::default()
        };

        primary_cb.begin_rendering(pass_info).unwrap();
        primary_cb
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: self.rcx().window.inner_size().into(),
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

        primary_cb
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap();

        unsafe {
            primary_cb.draw(3, 1, 0, 0).unwrap();
        }

        primary_cb.end_rendering().unwrap();
    }

    pub fn render(&mut self, vulkan: &Vulkan, states: &[AppState], resources: &Resources) {
        let rcx = self.rcx.as_mut().unwrap();

        let window_size = rcx.window.inner_size();
        let game_resolution = [
            window_size.width / RENDER_RES_RATIO[0],
            window_size.height / RENDER_RES_RATIO[1],
        ];

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
            rcx.color_image = ImageView::new_default(
                Image::new(
                    vulkan.memory_allocator.clone(),
                    ImageCreateInfo {
                        image_type: ImageType::Dim2d,
                        format: rcx.images[0].format(),
                        extent: [game_resolution[0], game_resolution[1], 1],
                        usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default(),
                )
                .unwrap(),
            )
            .unwrap();
            rcx.depth_image = ImageView::new_default(
                Image::new(
                    vulkan.memory_allocator.clone(),
                    ImageCreateInfo {
                        image_type: ImageType::Dim2d,
                        format: Format::D32_SFLOAT,
                        extent: [game_resolution[0], game_resolution[1], 1],
                        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT
                            | ImageUsage::TRANSIENT_ATTACHMENT,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default(),
                )
                .unwrap(),
            )
            .unwrap();
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

        let mut primary_cb = AutoCommandBufferBuilder::primary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let states_to_skip = states.len()
            - 1
            - states
                .iter()
                .rev()
                .take_while(|s| s.is_transparent())
                .count();
        let mut is_first = true;
        for state in states.iter().skip(states_to_skip) {
            self.render_state(
                vulkan,
                &mut primary_cb,
                state,
                resources,
                image_index,
                is_first,
            );
            if is_first {
                is_first = false
            }
        }

        let rcx = self.rcx.as_mut().unwrap();

        let command_buffer = primary_cb.build().unwrap();
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

pub mod game_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/game_object.vert"
    }
}

pub mod game_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/game_object.frag"
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
