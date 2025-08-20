use crate::graphics::{Graphics, /*PointLightRenderSystem,*/ Renderer, Vulkan};
use std::{error::Error, sync::Arc};
use vulkano::{
    VulkanLibrary,
    buffer::{
        BufferUsage,
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
    },
    command_buffer::allocator::{
        StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, QueueCreateInfo, QueueFlags,
        physical::PhysicalDeviceType,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator},
    swapchain::Surface,
};
use winit::event_loop::EventLoop;

impl Graphics {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let vulkan = Vulkan::init(event_loop)?;

        let renderer = Renderer::new(&vulkan);

        Ok(Graphics { vulkan, renderer })
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
                    enabled_features: DeviceFeatures {
                        descriptor_indexing: true,
                        shader_sampled_image_array_non_uniform_indexing: true,
                        runtime_descriptor_array: true,
                        descriptor_binding_variable_descriptor_count: true,
                        dynamic_rendering: true,
                        image_view_format_swizzle: true,
                        ..DeviceFeatures::empty()
                    },
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
            StandardCommandBufferAllocatorCreateInfo {
                secondary_buffer_count: 32,
                ..Default::default()
            },
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
