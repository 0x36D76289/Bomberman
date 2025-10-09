use glam::{Vec2, Vec4};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::{
    app_state::{AppState, KeyMap},
    audio::{AudioManager, BackgroundMusic, audio_manager},
    game::{game_settings::GameSettings, game_state::GameState, resources::Resources},
    ui::{UiState, canvas::Canvas, ui_state::UIPage},
};

impl UiState {
    pub fn main_menu() -> Self {
        let title = Canvas {
            center: Vec2::new(0.0, -0.3),
            text: Some("BOMBERMAN".to_string()),
            text_color: Some(Vec4::ONE),
            text_size: Some(2.0),
            ..Default::default()
        };

        let text1 = Canvas {
            center: Vec2::new(0.0, 0.2),
            text: Some("Press enter to play!".to_string()),
            text_color: Some(Vec4::ONE),
            text_size: Some(0.8),
            ..Default::default()
        };

        Self {
            canvases: vec![title, text1],
            buttons: Vec::new(),
            is_transparent: false,
            selected: 0,
            page: UIPage::MainMenu,
        }
    }
    pub fn main_menu_tick(
        &self,
        keys: &KeyMap,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        if let Some(state) = keys.get(&PhysicalKey::Code(KeyCode::Enter))
            && state.is_pressed()
        {
            // INFO: Le jeu devrait demarrer sa musique je pense

            // Changer la musique pour le jeu
            audio_manager.play_background_music(BackgroundMusic::Game);
            return (
                // TODO: replace player count with value gotten from settings/previous ui
                Some(AppState::Ui(UiState::game_settings(2))),
                0,
            );
        }
        if let Some(state) = keys.get(&PhysicalKey::Code(KeyCode::Backspace))
            && state.is_pressed()
        {
            return (Some(AppState::Ui(UiState::settings())), 0);
        }
        (None, 0)
    }
}
