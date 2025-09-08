use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    audio::{AudioManager, BackgroundMusic},
    game::{arena_state::ArenaState, game_settings::GameSettings, resources::Resources},
    input::input::Input,
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
            ..Default::default()
        };

        let mut resume = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, -0.2),
                text: Some("Resume".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.6),
                ..Default::default()
            },
            outline_color: None,
            neighbors: ButtonNeighbors {
                up: 0,
                down: 1,
                left: 0,
                right: 0,
            },
            selected_text_color: Some(Vec4::ONE.with_z(0.0)),
            ..Default::default()
        };
        resume.toggle();

        let restart = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.0),
                text: Some("Restart".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.6),
                ..Default::default()
            },
            outline_color: None,
            neighbors: ButtonNeighbors {
                up: 0,
                down: 2,
                left: 1,
                right: 1,
            },
            selected_text_color: Some(Vec4::ONE.with_z(0.0)),
            ..Default::default()
        };
        let menu = Button {
            canvas: Canvas {
                center: Vec2::new(0.0, 0.2),
                text: Some("Menu".to_string()),
                text_color: Some(Vec4::ONE),
                text_size: Some(1.6),
                ..Default::default()
            },
            outline_color: None,
            neighbors: ButtonNeighbors {
                up: 1,
                down: 2,
                left: 2,
                right: 2,
            },
            selected_text_color: Some(Vec4::ONE.with_z(0.0)),
            ..Default::default()
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
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        if self.button_inputs(inputs) {
            return match self.selected {
                0 => (None, 1), // Resume
                1 => {
                    // Restart
                    audio_manager.play_background_music(BackgroundMusic::Game);
                    (
                        Some(AppState::Arena(
                            ArenaState::default_state(resources, GameSettings::default().unwrap())
                                .unwrap(),
                        )),
                        2,
                    )
                }
                _ => {
                    // Menu
                    audio_manager.play_background_music(BackgroundMusic::Menu);
                    (None, 2)
                }
            };
        }
        (None, 0)
    }
}
