use crate::{
    app_state::{AppState, Ctx, POP_TO_MENU, Transition, replace_or_menu},
    graphics::text::FontId,
    settings::save::SaveState,
    ui::{
        UiState,
        draw::TextStyle,
        layout::Rect,
        style,
        textures::Icon,
        ui::id,
        widgets::{Action, ButtonStyle},
    },
};
use glam::{Vec2, Vec4};

pub fn tick(state: &mut UiState, ctx: &mut Ctx, run: SaveState) -> Transition {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();
    ui.fill(Vec4::new(0.3, 0.02, 0.05, 0.7));

    ui.title("GAME OVER", Vec2::new(screen.x / 2.0, 330.0), 112.0);

    ui.dl.text(
        &format!("Score final : {}", run.score),
        Vec2::new(screen.x / 2.0, 470.0),
        TextStyle::new(FontId::Title, 44.0, style::WHITE).outline(style::INK, 4.0),
    );
    ui.dl.text(
        &format!("Tu as atteint le niveau {}", run.level),
        Vec2::new(screen.x / 2.0, 530.0),
        TextStyle::new(FontId::Body, 30.0, style::WHITE),
    );

    let buttons = Rect::new((screen.x - 860.0) / 2.0, 640.0, 860.0, 90.0).columns(2, 40.0);
    let retry = SaveState {
        lives: 3,
        score: 0,
        powers: Default::default(),
        ..run
    };
    let mut transition = (None, 0);
    ui.default_focus(id("retry"));
    if ui
        .button(
            id("menu"),
            buttons[0],
            "Menu principal",
            Some(Icon::Home),
            ButtonStyle::Normal,
        )
        .clicked
        || ui.back_pressed()
    {
        ctx.settings.save_progress(retry);
        transition = (None, POP_TO_MENU);
    }
    if ui
        .button(
            id("retry"),
            buttons[1],
            "Réessayer",
            Some(Icon::Return),
            ButtonStyle::Go,
        )
        .clicked
    {
        transition = replace_or_menu(AppState::level_start(ctx.settings, retry, false), 2);
    }
    ui.prompts(&[(Action::Navigate, "Déplacer"), (Action::Confirm, "Valider")]);
    ui.end(ctx.audio);
    transition
}
