use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    audio::AudioManager,
    game::{game_state::GameState, resources::Resources},
    input::input::Input,
    settings::settings::Settings,
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
    },
};

impl UiState {
    /// The main menu ui constructor
    pub fn main_menu() -> Self {
        let title = Canvas {
            center: Vec2::new(0.0, -0.4),
            text: Some("BOMBERMAN".to_string()),
            text_color: Some(Vec4::ONE),
            text_size: Some(2.0),
            ..Default::default()
        };

        let mut continue_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, -0.1),
                text: Some("Continue".to_string()),
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
        continue_button.toggle();

        let new_game_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.1),
                text: Some("New Game".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.2),
                ..Default::default()
            },
            neighbors: ButtonNeighbors {
                up: 0,
                down: 2,
                left: 1,
                right: 1,
            },
            selected_text_color: Some(Vec4::new(1.0, 1.0, 0.0, 1.0)),
            ..Default::default()
        };

        let multi_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.3),
                text: Some("Multiplayer".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.2),
                ..Default::default()
            },
            neighbors: ButtonNeighbors {
                up: 1,
                down: 3,
                left: 2,
                right: 2,
            },
            selected_text_color: Some(Vec4::new(1.0, 1.0, 0.0, 1.0)),
            ..Default::default()
        };
        let settings_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.5),
                text: Some("Settings".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.2),
                ..Default::default()
            },
            neighbors: ButtonNeighbors {
                up: 2,
                down: 4,
                left: 3,
                right: 3,
            },
            selected_text_color: Some(Vec4::new(1.0, 1.0, 0.0, 1.0)),
            ..Default::default()
        };
        let quit_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.7),
                text: Some("Quit Game".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.2),
                ..Default::default()
            },
            neighbors: ButtonNeighbors {
                up: 3,
                down: 4,
                left: 4,
                right: 4,
            },
            selected_text_color: Some(Vec4::new(1.0, 1.0, 0.0, 1.0)),
            ..Default::default()
        };

        Self {
            canvases: vec![title],
            buttons: vec![
                continue_button,
                new_game_button,
                multi_button,
                settings_button,
                quit_button,
            ],
            selected: 0,
            render_info: Default::default(),
        }
    }

    /// The main menu ui tick function
    pub fn main_menu_tick(
        &mut self,
        inputs: &Vec<Input>,
        audio_manager: &mut AudioManager,
        resources: &Resources,
        settings: &Settings,
    ) -> (Option<AppState>, u8) {
        if self.button_inputs(inputs) {
            return match self.selected {
                0 => {
                    // Continue
                    let save = settings.single_player_save;
                    audio_manager.play_background_music(crate::audio::BackgroundMusic::Game);
                    match GameState::new_campaign(save.level, save.lives) {
                        Some(game_state) => (Some(AppState::game(game_state)), 1),
                        None => {
                            println!(
                                "Error: Failed to load saved game. Starting new game selection."
                            );
                            (Some(AppState::level_select()), 1)
                        }
                    }
                }
                1 => (Some(AppState::level_select()), 1),
                2 => (
                    Some(AppState::game_settings(
                        resources,
                        settings.binds.len() as u8,
                    )),
                    0,
                ),
                3 => (Some(AppState::settings()), 0),
                4 => (None, 1), // Quit
                _ => (None, 0),
            };
        }
        (None, 0)
    }
}
