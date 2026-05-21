use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    audio::{AudioManager, BackgroundMusic},
    game::game_state::GameState,
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    settings::{save::GameDifficulty, settings::Settings},
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
    },
};

/// The amount of levels in the campaign
const LEVEL_COUNT: u32 = 2;

impl UiState {
    /// The level select page ui constructor
    pub fn level_select(difficulty: GameDifficulty) -> Self {
        let mut buttons = Vec::new();

        for i in 1..=LEVEL_COUNT {
            let y_pos = -0.3 + (i as f32 * 0.2);
            let button = Button {
                canvas: Canvas {
                    center: Vec2::new(0.0, y_pos),
                    text: Some(format!("Level {}", i)),
                    text_color: Some(Vec4::ONE),
                    text_size: Some(1.2),
                    ..Default::default()
                },
                neighbors: ButtonNeighbors {
                    up: if i == 1 { 1 } else { i - 2 } as usize,
                    down: if i == LEVEL_COUNT { LEVEL_COUNT - 1 } else { i } as usize,
                    left: (i - 1) as usize,
                    right: (i - 1) as usize,
                },
                selected_text_color: Some(Vec4::new(1.0, 1.0, 0.0, 1.0)),
                ..Default::default()
            };
            buttons.push(button);
        }
        buttons[0].toggle();

        let title = Canvas {
            center: Vec2::new(0.0, -0.4),
            text: Some("SELECT A LEVEL".to_string()),
            text_color: Some(Vec4::ONE),
            text_size: Some(2.0),
            ..Default::default()
        };

        let difficulty_label = Canvas {
            center: Vec2::new(0.0, -0.2),
            text: Some(format!("Difficulty: {}", difficulty.label())),
            text_color: Some(Vec4::new(0.9, 0.9, 0.9, 1.0)),
            text_size: Some(0.9),
            ..Default::default()
        };

        let background = Canvas {
            center: Vec2::ZERO,
            width: 2.0,
            height: 2.0,
            color: Vec4::new(0.1, 0.1, 0.1, 1.0),
            ..Default::default()
        };

        Self {
            canvases: vec![background, title, difficulty_label],
            buttons,
            selected: 0,
            render_info: Default::default(),
        }
    }

    /// The level select ui page tick function
    pub fn level_select_tick(
        &mut self,
        inputs: &Vec<Input>,
        audio_manager: &mut AudioManager,
        difficulty: GameDifficulty,
        settings: &mut Settings,
    ) -> (Option<AppState>, u8) {
        if self.button_inputs(inputs) {
            let selected_level = self.selected as u32 + 1;
            audio_manager.play_background_music(BackgroundMusic::Game);

            return match GameState::new_campaign(selected_level, 3, 0, difficulty) {
                Some(game_state) => {
                    settings.single_player_save.level = selected_level;
                    settings.single_player_save.lives = 3;
                    settings.single_player_save.score = 0;
                    settings.single_player_save.difficulty = difficulty;
                    settings.save();
                    (Some(AppState::game(game_state)), 1)
                }
                None => {
                    println!(
                        "Error: Failed to load campaign level {}. Returning to menu.",
                        selected_level
                    );
                    audio_manager.play_background_music(BackgroundMusic::Menu);
                    (Some(AppState::main_menu()), 1)
                }
            };
        }
        if MenuInput::menu_back(inputs) == InputState::Pressed {
            return (Some(AppState::difficulty_select()), 1);
        }
        (None, 0)
    }
}
