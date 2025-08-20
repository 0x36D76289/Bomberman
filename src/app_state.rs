use std::{collections::HashMap, sync::Arc};

use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use winit::{event::ElementState, keyboard::PhysicalKey};

use crate::{
    game::game_state::GameState,
    graphics::{Renderer, Vulkan},
    ui::UiState,
};

pub type KeyMap = HashMap<PhysicalKey, ElementState>;

pub enum AppState {
    Game(GameState),
    Ui(UiState),
}

impl AppState {
    pub fn render(&self, renderer: &Renderer, vulkan: &Vulkan) -> Arc<SecondaryAutoCommandBuffer> {
        match self {
            AppState::Game(game_state) => game_state.render(vulkan, renderer),
            AppState::Ui(_) => !todo!("implement ui render system"),
        }
    }

    pub fn tick(
        &mut self,
        delta: f32,
        input_state: &KeyMap,
        window_size: (u32, u32),
    ) -> (Option<AppState>, u8) {
        match self {
            AppState::Game(game_state) => game_state.tick(delta, input_state, window_size),
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
