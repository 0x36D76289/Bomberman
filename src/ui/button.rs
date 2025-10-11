use glam::{Vec2, Vec4};

use crate::{
    graphics::object::TextureIndex,
    ui::{canvas::Canvas, consts::OUTLINE_WIDTH},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonNeighbors {
    pub up: usize,
    pub down: usize,
    pub left: usize,
    pub right: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Button {
    pub canvas: Canvas,
    pub outline_color: Option<Vec4>,
    pub neighbors: ButtonNeighbors,
    pub selected_color: Vec4,
    pub selected_text_color: Option<Vec4>,
    pub selected_texture: Option<TextureIndex>,
}

impl Button {
    pub fn toggle(&mut self) {
        std::mem::swap(&mut self.canvas.color, &mut self.selected_color);
        std::mem::swap(&mut self.canvas.text_color, &mut self.selected_text_color);
        std::mem::swap(&mut self.canvas.texture, &mut self.selected_texture);
    }

    pub fn generate_canvases(&self, ratio: f32) -> [Option<Canvas>; 2] {
        if let Some(outline_color) = self.outline_color {
            let bg = Canvas {
                color: outline_color,
                texture: None,
                text: None,
                text_color: None,
                text_size: None,
                ..self.canvas
            };
            let fg = Canvas {
                width: self.canvas.width - OUTLINE_WIDTH / ratio.max(1.0),
                height: self.canvas.height - OUTLINE_WIDTH / (1.0 / ratio).max(1.0),
                ..self.canvas.clone()
            };
            [Some(bg), Some(fg)]
        } else {
            [Some(self.canvas.clone()), None]
        }
    }

    pub fn with_neighbors(mut self, neighbors: ButtonNeighbors) -> Self {
        self.neighbors = neighbors;
        self
    }
    pub fn with_pos(mut self, pos: Vec2) -> Self {
        self.canvas.center = pos;
        self
    }
}
