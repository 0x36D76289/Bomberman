pub mod camera;
pub mod init;
pub mod light;
pub mod model;
pub mod object;
pub mod renderer;
pub mod systems;

use crate::graphics::systems::{game_object_system::GameObjectSystem, point_light_system::PointLightSystem};

pub use {
    camera::Camera,
    init::window_size_dependent_setup,
    light::Light,
    model::Model,
    object::{GameObject, Transform},
    renderer::Renderer,
};

use std::{hash::Hash, sync::Arc, time::Instant};
use systems::GlobalUbo;
use vulkano::{
    buffer::{BufferContents, allocator::SubbufferAllocator},
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{Device, Queue},
    instance::Instance,
    memory::allocator::StandardMemoryAllocator,
    pipeline::{GraphicsPipeline, graphics::vertex_input::Vertex},
    render_pass::{Framebuffer, RenderPass},
    shader::EntryPoint,
    swapchain::Swapchain,
    sync::GpuFuture,
};
use winit::window::Window;

pub struct Graphics {
    pub vulkan: Vulkan,
    pub renderer: Renderer,
    pub game_object_system: GameObjectSystem,
    pub point_light_system: PointLightSystem
}

pub struct Vulkan {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub uniform_buffer_allocator: SubbufferAllocator,
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

pub struct TimeInfo {
    pub dt: f32,
    pub avg_fps: f32,
    time: Instant,
    dt_sum: f32,
    frame_count: f32,
}

#[derive(BufferContents, Vertex, Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32B32_SFLOAT)]
    #[name("in_position")]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    #[name("in_normal")]
    pub normal: [f32; 3],

    #[format(R32G32_SFLOAT)]
    #[name("in_uv")]
    pub uv: [f32; 2],
}

impl PartialEq for MyVertex {
    fn eq(&self, other: &Self) -> bool {
        let iter_self = self
            .position
            .iter()
            .chain(self.normal.iter())
            .chain(self.uv.iter());
        let iter_other = other
            .position
            .iter()
            .chain(other.normal.iter())
            .chain(other.uv.iter());
        for (s, o) in iter_self.zip(iter_other) {
            if s.to_bits() != o.to_bits() {
                return false;
            }
        }
        true
    }
}

impl Eq for MyVertex {}

impl Hash for MyVertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for v in self
            .position
            .iter()
            .chain(self.normal.iter())
            .chain(self.uv.iter())
        {
            v.to_bits().hash(state);
        }
    }
}
