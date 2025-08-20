use crate::{
    app_state::AppState,
    graphics::{MyVertex, TimeInfo, Vulkan},
};
use egui_winit_vulkano::{Gui, GuiConfig};
use std::{sync::Arc, time::Instant};
use vulkano::{
    Validated, VulkanError,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderingAttachmentInfo, RenderingInfo,
        SubpassContents,
    },
    descriptor_set::layout::DescriptorBindingFlags,
    format::{ClearValue, Format},
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
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            subpass::{PipelineRenderingCreateInfo, PipelineSubpassType},
            vertex_input::{Vertex, VertexDefinition},
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
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

pub struct Renderer {
    rcx: Option<RenderContext>,
    pub world_pipeline: Option<Arc<GraphicsPipeline>>,
    pub gui: Option<Gui>,
    pub sampler: Arc<Sampler>,
}

pub struct RenderContext {
    pub window: Arc<Window>,
    pub surface: Arc<Surface>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<ImageView>>,
    pub depth_image: Arc<ImageView>,
    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub time_info: TimeInfo,
}

impl Renderer {
    pub fn new(vulkan: &Vulkan) -> Self {
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
            world_pipeline: None,
            gui: None,
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

        let window_size = window.inner_size();

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
                    image_extent: window_size.into(),
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

        let depth_image = ImageView::new_default(
            Image::new(
                vulkan.memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D32_SFLOAT,
                    extent: images[0].extent(),
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
            surface,
            swapchain,
            images,
            depth_image,
            recreate_swapchain,
            previous_frame_end,
            time_info,
        })
    }

    pub fn create_world_pipeline(&mut self, vulkan: &Vulkan) {
        let vertex_shader = vs::load(vulkan.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fragment_shader = fs::load(vulkan.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = MyVertex::per_vertex().definition(&vertex_shader).unwrap();
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

        self.world_pipeline = Some(
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

    pub fn create_gui(&mut self, event_loop: &ActiveEventLoop, vulkan: &Vulkan) {
        let rcx = self.rcx();

        self.gui = Some(Gui::new(
            event_loop,
            rcx.surface.clone(),
            vulkan.queue.clone(),
            rcx.swapchain.image_format(),
            GuiConfig::default(),
        ))
    }

    pub fn update_gui_event(&mut self, event: &WindowEvent) -> bool {
        self.gui.as_mut().unwrap().update(event)
    }

    pub fn rcx(&self) -> &RenderContext {
        self.rcx.as_ref().unwrap()
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

    pub fn render(&mut self, vulkan: &Vulkan, states: &Vec<AppState>) {
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
            rcx.depth_image = ImageView::new_default(
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
            CommandBufferUsage::SimultaneousUse,
        )
        .unwrap();

        let mut color_attachment =
            RenderingAttachmentInfo::image_view(rcx.images[image_index as usize].clone());
        color_attachment.load_op = AttachmentLoadOp::Clear;
        color_attachment.store_op = AttachmentStoreOp::Store;
        color_attachment.clear_value = Some(ClearValue::Float([0.0, 0.0, 0.0, 0.0]));

        let mut depth_attachment = RenderingAttachmentInfo::image_view(rcx.depth_image.clone());
        depth_attachment.load_op = AttachmentLoadOp::Clear;
        depth_attachment.clear_value = Some(ClearValue::DepthStencil((1.0, 0)));

        let world_pass_info = RenderingInfo {
            color_attachments: vec![Some(color_attachment)],
            depth_attachment: Some(depth_attachment),
            layer_count: 1,
            contents: SubpassContents::SecondaryCommandBuffers,
            ..Default::default()
        };

        primary_cb.begin_rendering(world_pass_info).unwrap();
        // primary_cb.bind_pipeline_graphics(self.world_pipeline.clone()).unwrap();

        //TODO: all states from last backwards until no transparency
        let state = states.last().unwrap();
        let secondary_cb = state.render(&self, vulkan);

        let rcx = self.rcx.as_mut().unwrap();

        primary_cb.execute_commands(secondary_cb).unwrap();

        primary_cb.end_rendering().unwrap();

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

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/game_object.vert"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/game_object.frag"
    }
}
