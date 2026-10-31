use crate::{
    app_state::Ctx,
    ui::{UiState, ui::Ui},
};

pub mod credits;
pub mod cutscene;
pub mod game_over;
pub mod hud;
pub mod lobby;
pub mod main_menu;
pub mod mp_end;
pub mod new_game;
pub mod pause;
pub mod settings;
pub mod stage_clear;
pub mod stage_map;

pub fn begin<'a>(state: &'a mut UiState, ctx: &Ctx) -> Ui<'a> {
    Ui::begin(state, ctx.ui, ctx.resources, ctx.screen, ctx.delta)
}
