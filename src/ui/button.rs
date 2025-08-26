use glam::Vec4;

use crate::{graphics::object::TextureIndex, ui::canvas::Canvas};

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
    pub neighbors: ButtonNeighbors,
    pub selected_color: Vec4,
    pub selected_text_color: Option<Vec4>,
    pub selected_texture: Option<TextureIndex>,
    // TODO: neighbors
}

impl Button {
    pub fn toggle(&mut self) {
        std::mem::swap(&mut self.canvas.color, &mut self.selected_color);
        std::mem::swap(&mut self.canvas.text_color, &mut self.selected_text_color);
        std::mem::swap(&mut self.canvas.texture, &mut self.selected_texture);
    }
}
