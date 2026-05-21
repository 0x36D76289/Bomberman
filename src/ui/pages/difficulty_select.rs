use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    audio::AudioManager,
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    settings::save::GameDifficulty,
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
    },
};

impl UiState {
    /// The difficulty select page ui constructor
    pub fn difficulty_select() -> Self {
        let title = Canvas {
            center: Vec2::new(0.0, -0.4),
            text: Some("SELECT DIFFICULTY".to_string()),
            text_color: Some(Vec4::ONE),
            text_size: Some(2.0),
            ..Default::default()
        };

        let background = Canvas {
            center: Vec2::ZERO,
            width: 2.0,
            height: 2.0,
            color: Vec4::new(0.1, 0.1, 0.1, 1.0),
            ..Default::default()
        };

        let mut easy_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, -0.05),
                text: Some("Easy".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.4),
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
        easy_button.toggle();

        let normal_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.15),
                text: Some("Normal".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.4),
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

        let hard_button = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.35),
                text: Some("Hard".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.4),
                ..Default::default()
            },
            neighbors: ButtonNeighbors {
                up: 1,
                down: 2,
                left: 2,
                right: 2,
            },
            selected_text_color: Some(Vec4::new(1.0, 1.0, 0.0, 1.0)),
            ..Default::default()
        };

        Self {
            canvases: vec![background, title],
            buttons: vec![easy_button, normal_button, hard_button],
            selected: 0,
            render_info: Default::default(),
        }
    }

    /// The difficulty select ui page tick function
    pub fn difficulty_select_tick(
        &mut self,
        inputs: &Vec<Input>,
        _audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        if self.button_inputs(inputs) {
            let difficulty = match self.selected {
                0 => GameDifficulty::Easy,
                1 => GameDifficulty::Normal,
                2 => GameDifficulty::Hard,
                _ => GameDifficulty::Normal,
            };
            return (Some(AppState::level_select(difficulty)), 1);
        }

        if MenuInput::menu_back(inputs) == InputState::Pressed {
            return (Some(AppState::main_menu()), 1);
        }

        (None, 0)
    }
}
