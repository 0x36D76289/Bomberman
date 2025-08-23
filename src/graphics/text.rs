use std::sync::Arc;

use glam::Vec2;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

use crate::graphics::GuiVertex;

pub struct TextRenderer {
    pub characters: [Character; 128],
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Character {
    pub top_left: Vec2,
    pub top_right: Vec2,
    pub bottom_left: Vec2,
    pub bottom_right: Vec2,
}

const ATLAS_WIDTH: u32 = 126;
const ATLAS_HEIGHT: u32 = 63;
const CHAR_WIDTH: u32 = 7;
const CHAR_HEIGHT: u32 = 9;
const COL_NUMBER: u32 = ATLAS_WIDTH / CHAR_WIDTH;

impl TextRenderer {
    pub fn new() -> Self {
        let mut characters = [Character::default(); 128];

        for i in 0..96 {
            let row = i as u32 / COL_NUMBER;
            let col = i as u32 % COL_NUMBER;

            let character = Character {
                top_left: Vec2::new(
                    (col * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32,
                    (row * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
                ),
                top_right: Vec2::new(
                    (col * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32,
                    ((row + 1) * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
                ),
                bottom_left: Vec2::new(
                    ((col + 1) * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32,
                    (row * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
                ),
                bottom_right: Vec2::new(
                    ((col + 1) * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32,
                    ((row + 1) * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
                ),
            };

            characters[i + 32] = character;
        }

        Self { characters }
    }

    pub fn render_str(
        &self,
        s: &str,
        text_size: f32,
        position: Vec2,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Subbuffer<[GuiVertex]> {
        let char_screen_width = text_size * (CHAR_WIDTH as f32 / CHAR_HEIGHT as f32) * 0.05;
        let char_screen_height = text_size * (CHAR_HEIGHT as f32 / CHAR_WIDTH as f32) * 0.05;
        let spacing = 0.9;

        let text_width = s.len() as f32 * char_screen_width * spacing;
        let mut x = position.x - text_width / 2.0;
        let y = position.y - char_screen_height / 2.0;

        let mut vertices = Vec::new();
        for c in s.chars() {
            let c = match c as u8 {
                32..127 => self.characters[c as usize],
                _ => self.characters[127],
            };

            let left = x + char_screen_width;
            let right = x;
            let top = y;
            let bottom = y + char_screen_height;
            let top_left = GuiVertex {
                position: [left, top],
                uv: c.bottom_left.into(),
            };
            let bottom_left = GuiVertex {
                position: [left, bottom],
                uv: c.bottom_right.into(),
            };
            let bottom_right = GuiVertex {
                position: [right, bottom],
                uv: c.top_right.into(),
            };
            let top_right = GuiVertex {
                position: [right, top],
                uv: c.top_left.into(),
            };
            vertices.extend_from_slice(&[
                top_right,
                top_left,
                bottom_left,
                bottom_left,
                bottom_right,
                top_right,
            ]);

            x += char_screen_width * spacing
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
}
