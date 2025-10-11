use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    audio::{AudioManager, BackgroundMusic},
    game::{game_state::GameState, resources::Resources},
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
        ui_state::UIPage,
    },
};

impl UiState {
    pub fn pause() -> Self {
        let shadow = Canvas {
            center: Vec2::ZERO,
            width: 0.5,
            height: 0.5,
            color: Vec4::ZERO.with_w(0.6),
            texture: None,
            text: None,
            text_color: None,
            text_size: None,
        };

        let mut resume = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, -0.2),
                width: 0.0,
                height: 0.0,
                color: Vec4::ZERO,
                texture: None,
                text: Some("Resume".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.6),
            },
            outline_color: None,
            neighbors: ButtonNeighbors {
                up: 0,
                down: 1,
                left: 0,
                right: 0,
            },
            selected_color: Vec4::ZERO,
            selected_text_color: Some(Vec4::ONE.with_z(0.0)),
            selected_texture: None,
        };
        resume.toggle();

        let restart = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.0),
                width: 0.0,
                height: 0.0,
                color: Vec4::ZERO,
                texture: None,
                text: Some("Restart".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.6),
            },
            outline_color: None,
            neighbors: ButtonNeighbors {
                up: 0,
                down: 2,
                left: 1,
                right: 1,
            },
            selected_color: Vec4::ZERO,
            selected_text_color: Some(Vec4::ONE.with_z(0.0)),
            selected_texture: None,
        };
        let menu = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.2),
                width: 0.0,
                height: 0.0,
                color: Vec4::ZERO,
                texture: None,
                text: Some("Menu".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.6),
            },
            outline_color: None,
            neighbors: ButtonNeighbors {
                up: 1,
                down: 2,
                left: 2,
                right: 2,
            },
            selected_color: Vec4::ZERO,
            selected_text_color: Some(Vec4::ONE.with_z(0.0)),
            selected_texture: None,
        };

        Self {
            canvases: vec![shadow],
            buttons: vec![resume, restart, menu],
            is_transparent: true,
            selected: 0,
            page: UIPage::Pause,
        }
    }

    pub fn pause_tick(
        &mut self,
        inputs: &Vec<Input>,
        _resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        if self.button_inputs(inputs) {
            return match self.selected {
                0 => (None, 1), // Resume
                1 => {
                    // Restart
                    audio_manager.play_background_music(BackgroundMusic::Game);
                    if let Some(game_state) = GameState::new_campaign(1, 3) {
                        (Some(AppState::Game(game_state)), 2)
                    } else {
                        println!(
                            "Error: Failed to load campaign level 1 for restart. Returning to menu."
                        );
                        audio_manager.play_background_music(BackgroundMusic::Menu);
                        (Some(AppState::Ui(UiState::main_menu())), 2)
                    }
                }
                _ => {
                    // Menu
                    audio_manager.play_background_music(BackgroundMusic::Menu);
                    (Some(AppState::Ui(UiState::main_menu())), 2)
                }
            };
        }
        if inputs.menu_back() == InputState::Pressed {
            return (None, 1);
        }
        (None, 0)
    }
}
