use std::collections::HashMap;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use winit::{event::ElementState, keyboard::PhysicalKey};

use crate::{game::game_state::GameState, graphics::Graphics, input::input::Input, ui::UiState};

pub type KeyMap = HashMap<PhysicalKey, ElementState>;
pub type CommandBuffer = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;

pub enum AppState {
    Game(GameState),
    Ui(UiState),
}

impl AppState {
    pub fn render(&self, graphics: &Graphics, command_buffer: &mut CommandBuffer) {
        match self {
            AppState::Game(game_state) => game_state.render(graphics, command_buffer),
            AppState::Ui(_) => (),
        }
    }

    pub fn tick(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        window_size: (u32, u32),
    ) -> (Option<AppState>, u8) {
        match self {
            AppState::Game(game_state) => game_state.tick(delta, inputs, keys, window_size),
            AppState::Ui(_) => (None, 0),
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self {
            AppState::Game(_) => false,
            AppState::Ui(state) => state.is_transparent(),
        }
    }
}
