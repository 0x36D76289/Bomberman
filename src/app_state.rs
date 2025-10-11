use std::sync::Arc;

use vulkano::command_buffer::SecondaryAutoCommandBuffer;

use crate::{
    audio::AudioManager,
    game::{game_state::GameState, resources::Resources},
    graphics::{Renderer, Vulkan},
    input::{event::InputEvent, input::Input},
    settings::settings::Settings,
    ui::UiState,
};

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
        events: &Vec<InputEvent>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
        settings: &mut Settings,
        ratio: f32,
    ) -> (Option<AppState>, u8) {
        match self {
            AppState::Game(game_state) => game_state.tick(delta, inputs, resources, audio_manager),
            AppState::Ui(ui_state) => ui_state.tick(
                delta,
                inputs,
                events,
                resources,
                audio_manager,
                settings,
                ratio,
            ),
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self {
            AppState::Game(_) => false,
            AppState::Ui(state) => state.is_transparent(),
        }
    }
}
