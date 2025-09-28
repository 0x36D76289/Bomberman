use std::collections::HashMap;

use winit::{event::ElementState, keyboard::PhysicalKey};

use crate::{
    audio::AudioManager,
    game::{game_settings::GameSettings, game_state::GameState, resources::Resources},
    graphics::{StateRenderInfo, ui_state},
    input::input::Input,
    ui::{UiState, game_settings::UIGameSettings, ui_state::UIPage},
};

pub type KeyMap = HashMap<PhysicalKey, ElementState>;

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub game: Option<GameState>,
    pub ui: Option<UiState>,
}

impl AppState {
    pub fn game(game_state: GameState) -> Self {
        Self {
            game: Some(game_state),
            ..Default::default()
        }
    }

    pub fn ui(ui_state: UiState) -> Self {
        Self {
            ui: Some(ui_state),
            ..Default::default()
        }
    }

    pub fn with_game(self, game_state: GameState) -> Self {
        Self {
            game: Some(game_state),
            ..self
        }
    }

    pub fn with_ui(self, ui_state: UiState) -> Self {
        Self {
            ui: Some(ui_state),
            ..self
        }
    }

    pub fn tick(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        let (ret_state1, ret_pop1) = self.tick_game(delta, inputs, keys, resources, audio_manager);
        let (ret_state2, ret_pop2) = self.tick_ui(delta, inputs, keys, resources, audio_manager);
        (ret_state1.or(ret_state2), ret_pop1.max(ret_pop2))
    }

    fn tick_game(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        match &mut self.game {
            Some(game_state) => game_state.tick(delta, inputs, keys, resources, audio_manager),
            None => (None, 0),
        }
    }

    fn tick_ui(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        match &mut self.ui {
            None => (None, 0),
            Some(ui_state) => {
                let ret = ui_state.tick(delta, inputs, keys, resources, audio_manager);
                if let UIPage::GameSettings(game_settings) = &ui_state.page {
                    let game_state = self.game.take().unwrap_or(GameState::new_settings_preview(
                        GameSettings::default(),
                        resources,
                    ));
                    self.game = Some(game_state.update_from_ui_settings(game_settings, resources));
                }
                ret
            }
        }
    }

    pub fn is_transparent(&self) -> bool {
        self.game.is_none()
            && match &self.ui {
                Some(ui_state) => ui_state.is_transparent(),
                None => true,
            }
    }
}
