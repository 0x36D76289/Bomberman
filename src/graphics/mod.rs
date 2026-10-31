#[rustfmt::skip]
#[allow(unused_imports, clippy::all)]
pub mod animation;
pub mod draw;
pub mod game_state;
pub mod init;
pub mod light;
#[rustfmt::skip]
#[allow(unused_imports, dead_code, clippy::all)]
pub mod mesh;
pub mod model;
pub mod object;
pub mod renderer;
pub mod text;
pub mod texture;
pub mod transform;
pub mod ui_state;

pub use {
    light::LightInfo,
    model::Model,
    renderer::{Renderer, StateRenderInfo, game_vs::GamePush, game_vs::GlobalUbo},
};

use std::{hash::Hash, sync::Arc, time::Instant};
use vulkano::{
    buffer::{BufferContents, allocator::SubbufferAllocator},
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{Device, Queue},
    instance::Instance,
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::vertex_input::Vertex,
};

pub struct Graphics {
    pub vulkan: Vulkan,
    pub renderer: Renderer,
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

pub struct TimeInfo {
    pub dt: f32,
    time: Instant,
}

#[derive(BufferContents, Vertex, Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct GameVertex {
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

#[derive(BufferContents, Vertex, Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct AnimationVertex {
    #[format(R32G32B32_SFLOAT)]
    #[name("in_position")]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    #[name("in_normal")]
    pub normal: [f32; 3],

    #[format(R32G32_SFLOAT)]
    #[name("in_uv")]
    pub uv: [f32; 2],

    #[format(R32G32B32A32_UINT)]
    #[name("in_joint_indices")]
    pub joint_ids: [u32; 4],

    #[format(R32G32B32A32_SFLOAT)]
    #[name("in_joint_weights")]
    pub joint_weights: [f32; 4],
}

#[derive(BufferContents, Vertex, Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct GuiVertex {
    #[format(R32G32_SFLOAT)]
    #[name("in_position")]
    pub position: [f32; 2],

    #[format(R32G32_SFLOAT)]
    #[name("in_uv")]
    pub uv: [f32; 2],
}

#[derive(BufferContents, Vertex, Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct UiVertex {
    #[format(R32G32_SFLOAT)]
    #[name("in_position")]
    pub position: [f32; 2],

    #[format(R32G32_SFLOAT)]
    #[name("in_uv")]
    pub uv: [f32; 2],

    #[format(R32G32B32A32_SFLOAT)]
    #[name("in_color")]
    pub color: [f32; 4],

    #[format(R32_SINT)]
    #[name("in_tex")]
    pub tex: i32,
}

impl PartialEq for GameVertex {
    fn eq(&self, other: &Self) -> bool {
        for (s, o) in self
            .position
            .iter()
            .chain(self.normal.iter())
            .chain(self.uv.iter())
            .zip(
                other
                    .position
                    .iter()
                    .chain(other.normal.iter())
                    .chain(other.uv.iter()),
            )
        {
            if s.to_bits() != o.to_bits() {
                return false;
            }
        }
        true
    }
}

impl Eq for GameVertex {}

impl Hash for GameVertex {
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
