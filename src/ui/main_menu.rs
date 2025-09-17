use glam::{Vec2, Vec4};

use crate::{
    app_state::{AppState, KeyMap},
    audio::{AudioManager, BackgroundMusic},
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

        let mut campaign_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.2),
                text: Some("Campaign".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.2),
                ..Default::default()
            },
            neighbors: ButtonNeighbors {
                up: 0,
                down: 1,
                left: 0,
                right: 0,
            },
            selected_text_color: Some(Vec4::new(1.0, 1.0, 0.0, 1.0)),
            ..Default::default()
        };
        campaign_button.toggle();

        let multi_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.4),
                text: Some("Multiplayer".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.2),
                ..Default::default()
            },
            neighbors: ButtonNeighbors {
                up: 0,
                down: 1,
                left: 1,
                right: 1,
            },
            selected_text_color: Some(Vec4::new(1.0, 1.0, 0.0, 1.0)),
            ..Default::default()
        };

        Self {
            canvases: vec![title],
            buttons: vec![campaign_button, multi_button],
            is_transparent: false,
            selected: 0,
            page: UIPage::MainMenu,
        }
    }
    pub fn main_menu_tick(
        &self,
        keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        match keys.get(&PhysicalKey::Code(KeyCode::Enter)) {
            Some(state) if state.is_pressed() => {
                // Changer la musique pour le jeu
                audio_manager.play_background_music(BackgroundMusic::Game);

                (
                    //TODO: replace with safe variant
                    Some(AppState::Game(
                        GameState::default_state(resources, GameSettings::default().unwrap())
                            .unwrap(),
                    )),
                    0,
                )
            }
            _ => (None, 0),
        }
        (None, 0)
    }
}
