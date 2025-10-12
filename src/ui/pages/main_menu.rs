use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    audio::AudioManager,
    game::{game_state::GameState, resources::Resources},
    input::input::Input,
    settings::save::SaveState,
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
    },
};

impl UiState {
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
    pub fn main_menu_tick(
        &mut self,
        inputs: &Vec<Input>,
        audio_manager: &mut AudioManager,
        resources: &Resources,
    ) -> (Option<AppState>, u8) {
        if self.button_inputs(inputs) {
            println!("selected: {}", self.selected);
            //TODO: je comprends pas pourquoi on joue Game peu importe le setting
            audio_manager.play_background_music(crate::audio::BackgroundMusic::Game);
            return match self.selected {
                0 => {
                    let save = SaveState::load();
                    match GameState::new_campaign(save.level, save.lives) {
                        Some(game_state) => (Some(AppState::game(game_state)), 0),
                        None => {
                            println!("Error: Failed to load saved campaign level {}", save.level);
                            (
                                Some(AppState::game(
                                    GameState::new_campaign(1, 3)
                                        .expect("Could not load level 1 as fallback"),
                                )),
                                0,
                            )
                        }
                    }
                }
                1 => match GameState::new_campaign(1, 3) {
                    // creer un nouveau main menu
                    Some(game_state) => (Some(AppState::game(game_state)), 0),
                    None => {
                        println!("Error: Failed to load campaign level 1 for new game");
                        (None, 0)
                    }
                },
                // TODO: get settings player count
                2 => (Some(AppState::game_settings(resources, 2)), 0),
                3 => (Some(AppState::settings()), 0),
                4 => (None, 1),
                _ => (None, 0),
            };
        }
        (None, 0)
    }
}
