use std::{collections::HashMap, sync::Arc};

use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use winit::{event::ElementState, keyboard::PhysicalKey};

use crate::{
    game::{game_state::GameState, resources::Resources},
    graphics::{Renderer, Vulkan},
    input::input::Input,
    ui::UiState,
};

pub type KeyMap = HashMap<PhysicalKey, ElementState>;

#[derive(Debug, Clone)]
pub enum AppState {
    Game(GameState),
    Ui(UiState),
}

impl AppState {
    pub fn render(
        &self,
        renderer: &Renderer,
        vulkan: &Vulkan,
        resources: &Resources,
    ) -> Arc<SecondaryAutoCommandBuffer> {
        match self {
            AppState::Game(game_state) => game_state.render(vulkan, renderer, resources),
            AppState::Ui(ui_state) => ui_state.render(vulkan, renderer, resources),
        }
    }

    pub fn tick(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
    ) -> (Option<AppState>, u8) {
        match self {
            AppState::Game(game_state) => game_state.tick(delta, inputs, keys, resources),
            AppState::Ui(ui_state) => ui_state.tick(delta, inputs, keys, resources),
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self {
            AppState::Game(_) => false,
            AppState::Ui(state) => state.is_transparent(),
        }
    }
}
