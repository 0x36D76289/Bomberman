use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    audio::AudioManager,
    input::input::Input,
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
        ui_state::UIPage,
    },
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
        &mut self,
        inputs: &Vec<Input>,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        if self.button_inputs(inputs) {
            audio_manager.play_background_music(crate::audio::BackgroundMusic::Game);
            return match self.selected {
                0 => {
                    // Campaign
                    match crate::game::game_state::GameState::new_campaign(1, 3) {
                        Some(game_state) => (Some(AppState::Game(game_state)), 1),
                        None => {
                            println!("Error: Failed to load campaign level 1");
                            (None, 0)
                        }
                    }
                }
                1 => {
                    // Multiplayer
                    (Some(AppState::Ui(UiState::game_settings(2))), 1)
                }
                _ => (None, 0),
            };
        }
        (None, 0)
    }
}
