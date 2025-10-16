use crate::graphics::{GuiPush, GuiVertex, object::TextureIndex};
use glam::{Vec2, Vec4};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

#[derive(Debug, Default, Clone)]
/// Represents a rectangle canvas that can be drawn on the screen.\
/// Can optionally have a text that will be rendered at the center of the canvas.\
/// Can optionally have a texture that will replace the color of the canvas.\
pub struct Canvas {
    /// Center of the canvas (between \[-1, -1\] and \[1, 1\]).
    pub center: Vec2,
    /// Width of the canvas (between -1 and 1).
    pub width: f32,
    /// Height of the canvas (between -1 and 1).
    pub height: f32,
    /// Color of the canvas (RGBA format).
    pub color: Vec4,
    /// The index of the canvas texture.
    pub texture: Option<TextureIndex>,
    /// The text that will be rendered inside the canvas.
    pub text: Option<String>,
    /// The text color (RGBA format).
    pub text_color: Option<Vec4>,
    /// The size of the text, for reference the default size is 1.
    pub text_size: Option<f32>,
}

const VERTEX_UV: [[f32; 2]; 6] = [
    [1.0, 0.0],
    [0.0, 0.0],
    [0.0, 1.0],
    [0.0, 1.0],
    [1.0, 1.0],
    [1.0, 0.0],
];

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

    /// Creates a vertex buffer for the canvas rectangle
    pub fn into_vertex_buffer(
        &self,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Subbuffer<[GuiVertex]> {
        let mut vertices = Vec::new();

        for (position, uv) in self.vertex_positions().into_iter().zip(VERTEX_UV) {
            let vertex = GuiVertex {
                position: position.into(),
                uv,
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

    /// Creates the push constant for the canvas rectangle
    pub fn push_constant(&self) -> GuiPush {
        match self.texture {
            Some(index) => GuiPush {
                color: Vec4::ONE.into(),
                tex_index: index,
            },
            None => GuiPush {
                color: self.color.into(),
                tex_index: -1,
            },
        }
    }
}
