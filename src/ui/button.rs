use glam::{Vec2, Vec4};

use crate::{
    graphics::object::TextureIndex,
    ui::{canvas::Canvas, consts::OUTLINE_WIDTH},
};

/// The neighbors are the selected buttons when navigating a ui with direction keys
#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonNeighbors {
    /// The index of the button above
    pub up: usize,
    /// The index of the button below
    pub down: usize,
    /// The index of the button to the left
    pub left: usize,
    /// The index of the button to the right
    pub right: usize,
}

/// A Button is similar to a [Canvas] that can be acted on using inputs, it changes appearance when selected
#[derive(Debug, Clone, Default)]
pub struct Button {
    /// The button's appearance when unselected
    pub canvas: Canvas,
    /// The button's optional outline color (RGBA)
    pub outline_color: Option<Vec4>,
    /// The buttons which are selected when pressing direction keys while hovering over this one
    pub neighbors: ButtonNeighbors,
    /// The color the button takes when selected (RGBA)
    pub selected_color: Vec4,
    /// The color of the text when the button selected (RGBA)
    pub selected_text_color: Option<Vec4>,
    /// The texture of the button when selected
    pub selected_texture: Option<TextureIndex>,
}

impl Button {
    /// Used to select or deselect a button
    pub fn toggle(&mut self) {
        std::mem::swap(&mut self.canvas.color, &mut self.selected_color);
        std::mem::swap(&mut self.canvas.text_color, &mut self.selected_text_color);
        std::mem::swap(&mut self.canvas.texture, &mut self.selected_texture);
    }

    /// Generates canvases to be passed to the render function
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

    /// A chainable function to modify a button's neighbors at construction
    pub fn with_neighbors(mut self, neighbors: ButtonNeighbors) -> Self {
        self.neighbors = neighbors;
        self
    }
    /// A chainable function to modify a button's position at construction
    pub fn with_pos(mut self, pos: Vec2) -> Self {
        self.canvas.center = pos;
        self
    }
}
