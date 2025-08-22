use crate::graphics::{GuiPush, GuiVertex, object::TextureIndex};
use glam::{Vec2, Vec4};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

#[derive(Debug, Default, Clone)]
/// Represents a rectangle canvas that can be drawn on the screen.\
/// It has coordinates in screen space (between -1 and 1) and a RGBA color.\
/// It can also optionally have a text that will be centered and a texture that will replace its color
pub struct Canvas {
    pub center: Vec2,
    pub width: f32,
    pub height: f32,
    pub color: Vec4,
    pub text: Option<String>,
    pub texture: Option<TextureIndex>,
}

impl Canvas {
    fn vertex_positions(&self) -> [Vec2; 6] {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        let top_left = self.center + Vec2::new(-half_width, half_height);
        let top_right = self.center + Vec2::new(half_width, half_height);
        let bottom_left = self.center + Vec2::new(-half_width, -half_height);
        let bottom_right = self.center + Vec2::new(half_width, -half_height);

        [
            top_right,
            top_left,
            bottom_left,
            bottom_left,
            bottom_right,
            top_right,
        ]
    }

    pub fn into_vertex_buffer(
        &self,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Subbuffer<[GuiVertex]> {
        let mut vertices = Vec::new();

        for position in self.vertex_positions() {
            let vertex = GuiVertex {
                position: position.into(),
                uv: Vec2::ZERO.into(),
            };
            vertices.push(vertex);
        }

        Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap()
    }

    pub fn push_constant(&self) -> GuiPush {
        GuiPush {
            color: self.color.into(),
            tex_index: self.texture.unwrap_or(-1),
        }
    }
}
