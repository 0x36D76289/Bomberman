use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    audio::AudioManager,
    input::input::Input,
    settings::save::GameDifficulty,
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
    },
};

impl UiState {
    /// The game over screen ui page constructor
    pub fn game_over(score: u32) -> Self {
        let shadow = Canvas {
            center: Vec2::ZERO,
            width: 2.0,
            height: 2.0,
            color: Vec4::new(0.4, 0.0, 0.0, 0.7),
            ..Default::default()
        };

        let title = Canvas {
            center: Vec2::new(0.0, -0.3),
            text: Some("GAME OVER".to_string()),
            text_color: Some(Vec4::ONE),
            text_size: Some(2.0),
            ..Default::default()
        };

        let score_text = Canvas {
            center: Vec2::new(0.0, -0.05),
            text: Some(format!("SCORE: {}", score)),
            text_color: Some(Vec4::ONE),
            text_size: Some(1.2),
            ..Default::default()
        };

        let mut retry_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.1),
                text: Some("Retry".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.6),
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
        retry_button.toggle(); // Select by default

        let menu_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.3),
                text: Some("Main Menu".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.6),
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
            canvases: vec![shadow, title, score_text],
            buttons: vec![retry_button, menu_button],
            selected: 0,
            render_info: Default::default(),
        }
    }

    /// The game over ui page tick function
    pub fn game_over_tick(
        &mut self,
        inputs: &Vec<Input>,
        audio_manager: &mut AudioManager,
        difficulty: GameDifficulty,
    ) -> (Option<AppState>, u8) {
        if self.button_inputs(inputs) {
            return match self.selected {
                0 => {
                    // Retry
                    if let Some(game_state) =
                        crate::game::game_state::GameState::new_campaign(1, 3, 0, difficulty)
                    {
                        (Some(AppState::game(game_state)), 2)
                    } else {
                        println!(
                            "Error: Failed to load campaign level 1 for retry. Returning to menu."
                        );
                        audio_manager.play_background_music(crate::audio::BackgroundMusic::Menu);
                        (Some(AppState::main_menu()), 2)
                    }
                }
                _ => {
                    // Main Menu
                    audio_manager.play_background_music(crate::audio::BackgroundMusic::Menu);
                    (Some(AppState::main_menu()), 2)
                }
            };
        }
        (None, 0)
    }
}
