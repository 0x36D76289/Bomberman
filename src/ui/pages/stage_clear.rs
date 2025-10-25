use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    game::game_state::GameState,
    settings::settings::Settings,
    ui::{UiState, canvas::Canvas},
};

/// How long the stage clear should last for
pub const STAGE_CLEAR_DURATION: f32 = 2.0;

impl UiState {
    /// The stage clear ui page constructor: increments level and saves
    pub fn stage_clear(settings: &mut Settings, level: u32, lives: u32) -> Self {
        settings.single_player_save.level = level + 1;
        settings.single_player_save.lives = lives;
        settings.single_player_save.score = 0;
        // TODO: Implement scoring and carry it over
        println!("Stage clear! Saving progress for next level.");
        settings.save();

        let shadow = Canvas {
            center: Vec2::ZERO,
            width: 2.0,
            height: 2.0,
            color: Vec4::new(0.0, 0.0, 0.0, 0.7),
            ..Default::default()
        };

        let title = Canvas {
            center: Vec2::new(0.0, 0.0),
            text: Some(format!(
                "STAGE {} CLEAR",
                settings.single_player_save.level - 1
            )),
            text_color: Some(Vec4::ONE),
            text_size: Some(2.0),
            ..Default::default()
        };

        Self {
            canvases: vec![shadow, title],
            buttons: Vec::new(),
            selected: 0,
            render_info: Default::default(),
        }
    }

    /// The stage clear ui page tick function
    pub fn stage_clear_tick(
        &mut self,
        delta: f32,
        timer: &mut f32,
        next_level: &mut u32,
        lives: &mut u32,
    ) -> (Option<AppState>, u8) {
        *timer -= delta;
        if *timer <= 0.0 {
            let next_game_state = GameState::new_campaign(*next_level, *lives);
            let app_state = {
                match next_game_state {
                    Some(game_state) => AppState::game(game_state),
                    None => AppState::main_menu(),
                }
            };
            return (Some(app_state), 1);
        }

        (None, 0)
    }
}
