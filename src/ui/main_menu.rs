use glam::{Vec2, Vec4};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::{
    app_state::{AppState, KeyMap},
<<<<<<< HEAD
<<<<<<< HEAD
    audio::{AudioManager, BackgroundMusic},
    game::{arena_state::ArenaState, game_settings::GameSettings, resources::Resources},
=======
    game::{game_settings::GameSettings, game_state::GameState, resources::Resources},
>>>>>>> 081d3f4 (started ui for game settings selection)
=======
    audio::{AudioManager, BackgroundMusic, audio_manager},
    game::{game_settings::GameSettings, game_state::GameState, resources::Resources},
=======
    audio::{AudioManager, BackgroundMusic},
    game::{arena_state::ArenaState, game_settings::GameSettings, resources::Resources},
>>>>>>> 360c3aa (feat: Implement a simple single-player system, with enemies, and pathing)
>>>>>>> 71022cd (feat: Implement a simple single-player system, with enemies, and pathing)
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
<<<<<<< HEAD

    pub fn main_menu_tick(
        &self,
        keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        match keys.get(&PhysicalKey::Code(KeyCode::Enter)) {
            Some(state) if state.is_pressed() => {
                audio_manager.play_background_music(BackgroundMusic::Game);
                (
<<<<<<< HEAD
=======
<<<<<<< HEAD
                    // TODO: replace player count with value gotten from settings/previous ui
                    Some(AppState::Ui(UiState::game_settings(2))),
=======
>>>>>>> 71022cd (feat: Implement a simple single-player system, with enemies, and pathing)
                    Some(AppState::Arena(
                        ArenaState::default_state(resources, GameSettings::default().unwrap())
                            .unwrap(),
                    )),
<<<<<<< HEAD
=======
>>>>>>> 360c3aa (feat: Implement a simple single-player system, with enemies, and pathing)
>>>>>>> 71022cd (feat: Implement a simple single-player system, with enemies, and pathing)
                    0,
                )
            }
=======
    pub fn main_menu_tick(&self, keys: &KeyMap) -> (Option<AppState>, u8) {
        match keys.get(&PhysicalKey::Code(KeyCode::Enter)) {
            Some(state) if state.is_pressed() => (
                // TODO: replace player count with value gotten from settings/previous ui
                Some(AppState::Ui(UiState::game_settings(2))),
                0,
            ),
>>>>>>> 081d3f4 (started ui for game settings selection)
            _ => (None, 0),
        }
    }
}
