use crate::graphics::{
    systems::{game_object_system::GameObjectSystem, point_light_system::PointLightSystem, GlobalUbo}, Graphics, MyVertex, RenderContext, Renderer, TimeInfo, Vulkan
};
use std::{error::Error, sync::Arc, time::Instant};
use vulkano::{
    VulkanLibrary,
    buffer::{
        Buffer, BufferCreateInfo, BufferUsage,
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
    },
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceOwned, QueueCreateInfo, QueueFlags,
        physical::PhysicalDeviceType,
    },
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::EntryPoint,
    swapchain::{ColorSpace, PresentMode, Surface, Swapchain, SwapchainCreateInfo},
    sync::{self, GpuFuture},
};
use winit::{
    dpi::PhysicalSize,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

impl Graphics {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let vulkan = Vulkan::init(event_loop)?;

        Ok(Graphics {
            vulkan,
            renderer: Renderer::new(),
            game_object_system: GameObjectSystem::default(),
            point_light_system: PointLightSystem::default()
        })
    }
}

impl Vulkan {
    pub fn init(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let library = VulkanLibrary::new()?;

        let instance = {
            let required_extensions = Surface::required_extensions(event_loop)?;

            Instance::new(
                library,
                InstanceCreateInfo {
                    flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                    enabled_extensions: required_extensions,
                    ..Default::default()
                },
            )?
        };

        // selecting a physical device (eg. graphic card) and creating a Device and a queue from it that we will use to do all future operations
        let (device, queue) = {
            let device_extensions = DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::empty()
            };

            let (physical_device, queue_family_index) = instance
                .enumerate_physical_devices()?
                .filter(|p| p.supported_extensions().contains(&device_extensions))
                .filter_map(|p| {
                    p.queue_family_properties()
                        .iter()
                        .enumerate()
                        .position(|(i, q)| {
                            q.queue_flags.intersects(QueueFlags::GRAPHICS)
                                && p.presentation_support(i as u32, event_loop).unwrap()
                        })
                        .map(|i| (p, i as u32))
                })
                .min_by_key(|(p, _)| match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                    _ => 5,
                })
                .unwrap();

            println!(
                "Using physical device: {} (type: {:?})",
                physical_device.properties().device_name,
                physical_device.properties().device_type
            );

            let (device, mut queues) = Device::new(
                physical_device,
                DeviceCreateInfo {
                    enabled_extensions: device_extensions,
                    queue_create_infos: vec![QueueCreateInfo {
                        queue_family_index,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )?;

            (device, queues.next().unwrap())
        };

        // creating allocators for vulkan
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let uniform_buffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        Ok(Self {
            instance,
            device,
            queue,
            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            uniform_buffer_allocator,
        })
    }
}

impl RenderContext {
    pub fn init(event_loop: &ActiveEventLoop, vulkan: &Vulkan) -> Result<Self, Box<dyn Error>> {
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
                surface,
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

        let render_pass = vulkano::single_pass_renderpass!(
            vulkan.device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth_stencil: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            },
        )
        .unwrap();

        let framebuffers = window_size_dependent_setup(
            window_size,
            &images,
            &render_pass,
            &vulkan.memory_allocator,
        );

        let recreate_swapchain = false;
        let previous_frame_end = Some(sync::now(vulkan.device.clone()).boxed());

        let time_info = TimeInfo {
            time: Instant::now(),
            dt: 0.0,
            frame_count: 0.0,
            avg_fps: 0.0,
            dt_sum: 0.0,
        };

        Ok(RenderContext {
            window,
            swapchain,
            render_pass,
            framebuffers,
            recreate_swapchain,
            previous_frame_end,
            time_info,
        })
    }
}

// this function creates the framebuffers and the graphics pipeline, it is called when we create the window and when we resize it
pub fn window_size_dependent_setup(
    window_size: PhysicalSize<u32>,
    images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>,
    memory_allocator: &Arc<StandardMemoryAllocator>,
) -> (Vec<Arc<Framebuffer>>) {
    let device = memory_allocator.device();

    let depth_buffer = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
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

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    framebuffers
}
