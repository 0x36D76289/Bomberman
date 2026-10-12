use crate::{
    app_state::{AppState, Ctx, Transition},
    game::{map::map::Map, theme::Theme},
    graphics::text::FontId,
    settings::save::SaveState,
    ui::{
        UiState,
        draw::{Align, TextStyle},
        layout::Rect,
        style,
        textures::{Icon, UiTex},
        ui::id,
        widgets::{Action, ButtonStyle},
    },
};
use glam::{Vec2, Vec4};

const COLUMN_WIDTH: f32 = 560.0;

fn save_summary(save: &SaveState) -> (String, String) {
    (
        format!(
            "Niveau {} - {}",
            save.level,
            Map::level_info(save.level)
                .map(|(_, theme)| theme.display_name())
                .unwrap_or(Theme::Grass.display_name())
        ),
        format!(
            "{} - {} pts - {} {}",
            save.difficulty.label(),
            save.score,
            save.lives,
            if save.lives == 1 { "vie" } else { "vies" }
        ),
    )
}

pub fn tick(state: &mut UiState, ctx: &mut Ctx) -> Transition {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();
    let save = ctx.settings.single_player_save;

    ui.fill(Vec4::new(0.05, 0.02, 0.12, 0.25));

    let left = style::MARGIN + COLUMN_WIDTH / 2.0 + 40.0;

    ui.dl.rect(
        Rect::new(0.0, 0.0, left + COLUMN_WIDTH / 2.0 + 60.0, screen.y),
        Vec4::new(0.08, 0.04, 0.18, 0.55),
    );

    ui.title("BOMBERMAN", Vec2::new(left, 170.0), 108.0);
    ui.dl.text(
        "LA QUÊTE DE LA MÈCHE D'OR",
        Vec2::new(left, 262.0),
        TextStyle::new(FontId::Title, 28.0, style::WHITE).outline(style::INK, 3.0),
    );

    let mut transition = (None, 0);
    let buttons = [
        ("continue", "Continuer", Icon::Next),
        ("new", "Nouvelle partie", Icon::Singleplayer),
        ("multi", "Multijoueur", Icon::Multiplayer),
        ("settings", "Paramètres", Icon::Gear),
        ("credits", "Crédits", Icon::Trophy),
        ("quit", "Quitter", Icon::Power),
    ];

    let mut quitting = ui.state.memory_flag(id("quitting"));

    if !quitting {
        ui.default_focus(id(if save.is_some() { "continue" } else { "new" }));

        for ((name, text, icon), rect) in buttons.iter().zip(
            Rect::new(
                left - COLUMN_WIDTH / 2.0,
                330.0,
                COLUMN_WIDTH,
                6.0 * 92.0 - 16.0,
            )
            .rows(buttons.len(), 16.0),
        ) {
            let r = ui.button(
                id(name),
                rect,
                text,
                Some(*icon),
                match *name {
                    "continue" if save.is_none() => ButtonStyle::Disabled,
                    "continue" => ButtonStyle::Go,
                    "quit" => ButtonStyle::Danger,
                    _ => ButtonStyle::Normal,
                },
            );

            if *name == "continue"
                && r.focused
                && let Some(save) = &save
            {
                let (line1, line2) = save_summary(save);
                let card = Rect::new(rect.right() + 40.0, rect.y - 10.0, 520.0, rect.h + 20.0);

                ui.dl.frame(UiTex::Panel, card, Vec4::ONE, 0.5);
                ui.dl.text(
                    &line1,
                    Vec2::new(card.x + 30.0, card.y + 34.0),
                    TextStyle::new(FontId::Title, 28.0, style::TEXT_DARK).align(Align::Left),
                );
                ui.dl.text(
                    &line2,
                    Vec2::new(card.x + 30.0, card.y + 70.0),
                    TextStyle::new(FontId::Body, 24.0, style::TEXT_MUTED).align(Align::Left),
                );
            }
            if r.clicked {
                transition = match *name {
                    "continue" => (
                        save.and_then(|s| AppState::level_start(ctx.settings, s, false)),
                        0,
                    ),
                    "new" => (Some(AppState::new_game()), 0),
                    "multi" => (
                        Some(AppState::lobby(
                            ctx.resources,
                            ctx.inputs.len().min(8) as u8,
                        )),
                        0,
                    ),
                    "settings" => (Some(AppState::settings(false)), 0),
                    "credits" => (Some(AppState::credits()), 0),
                    _ => {
                        quitting = true;
                        (None, 0)
                    }
                };
            }
        }
        if ui.back_pressed() {
            quitting = true;
        }
    }

    ui.dl.text(
        "42 Paris de la secte du Rust",
        Vec2::new(style::MARGIN, screen.y - 52.0),
        TextStyle::new(FontId::Body, 22.0, style::fade(style::WHITE, 0.7)).align(Align::Left),
    );
    ui.prompts(&[(Action::Navigate, "Déplacer"), (Action::Confirm, "Valider")]);

    if quitting {
        match ui.dialog(
            "quit",
            "Quitter le jeu ?",
            "Ta progression est sauvegardée après chaque niveau.",
            "Quitter",
        ) {
            Some(true) => transition = (None, 1),
            Some(false) => quitting = false,
            None => (),
        }
    }
    ui.state.set_memory_flag(id("quitting"), quitting);
    ui.end(ctx.audio);
    transition
}
