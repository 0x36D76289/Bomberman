use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    game::game_state::GameState,
    settings::save::SaveState,
    ui::{UiState, canvas::Canvas, ui_state::UIPage},
};

const STAGE_CLEAR_DURATION: f32 = 2.0;

impl UiState {
    pub fn stage_clear(level: u32, lives: u32) -> Self {
        println!("Stage clear! Saving progress for next level.");
        let next_level_state = SaveState {
            level: level + 1,
            lives,
            score: 0, // TODO: Implement scoring and carry it over
        };
        next_level_state.save();

        let shadow = Canvas {
            center: Vec2::ZERO,
            width: 2.0,
            height: 2.0,
            color: Vec4::new(0.0, 0.0, 0.0, 0.7),
            ..Default::default()
        };

        let title = Canvas {
            center: Vec2::new(0.0, 0.0),
            text: Some(format!("STAGE {} CLEAR", level)),
            text_color: Some(Vec4::ONE),
            text_size: Some(2.0),
            ..Default::default()
        };

        Self {
            canvases: vec![shadow, title],
            buttons: Vec::new(),
            is_transparent: true,
            selected: 0,
            page: UIPage::StageClear {
                timer: STAGE_CLEAR_DURATION,
                next_level: level + 1,
                lives,
            },
        }
    }

    pub fn stage_clear_tick(&mut self, delta: f32) -> (Option<AppState>, u8) {
        if let UIPage::StageClear {
            timer,
            next_level,
            lives,
        } = &mut self.page
        {
            *timer -= delta;
            if *timer <= 0.0 {
                let next_game_state = GameState::new_campaign(*next_level, *lives);
                if let Some(game_state) = next_game_state {
                    return (Some(AppState::Game(game_state)), 1);
                } else {
                    println!("No more levels found. Returning to main menu.");
                    return (Some(AppState::Ui(UiState::main_menu())), 1);
                }
            }
        }
        (None, 0)
    }
}
