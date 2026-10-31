use crate::{
    app_state::{AppState, Ctx, Transition, replace_or_menu},
    game::{game_state::ClearStats, map::map::LEVEL_COUNT},
    graphics::text::FontId,
    settings::{save::SaveState, settings::Settings},
    ui::{
        UiState,
        draw::{Align, TextStyle},
        layout::Rect,
        pages::cutscene::{CutsceneNext, Scene},
        style,
        textures::{Icon, UiTex},
        ui::id,
        widgets::{Action, ButtonStyle},
    },
};
use glam::Vec2;

pub fn save(settings: &mut Settings, run: SaveState) {
    if run.level < LEVEL_COUNT {
        settings.save_progress(SaveState {
            level: run.level + 1,
            ..run
        });
    } else {
        settings.unlocked_level = LEVEL_COUNT;
        settings.single_player_save = None;
        settings.save();
    }
}

pub fn tick(
    state: &mut UiState,
    ctx: &mut Ctx,
    run: SaveState,
    stats: ClearStats,
    name: &str,
) -> Transition {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();

    ui.fill(style::fade(style::OVERLAY, 0.9));

    let panel = Rect::centered(screen / 2.0 + Vec2::new(0.0, 30.0), Vec2::new(820.0, 640.0));

    ui.panel(panel);
    ui.title(
        "NIVEAU TERMINÉ !",
        Vec2::new(screen.x / 2.0, panel.y - 34.0),
        72.0,
    );
    ui.dl.text(
        &format!("Niveau {} - {name}", run.level),
        Vec2::new(screen.x / 2.0, panel.y + 90.0),
        TextStyle::new(FontId::Title, 36.0, style::TEXT_DARK),
    );

    for (i, (label, value)) in [
        ("Points du niveau", stats.stage_points),
        ("Bonus de fin", stats.clear_bonus),
        ("Bonus de temps", stats.time_bonus),
    ]
    .iter()
    .enumerate()
    {
        let y = panel.y + 170.0 + i as f32 * 70.0;

        ui.dl.text(
            label,
            Vec2::new(panel.x + 80.0, y),
            TextStyle::new(FontId::Title, 34.0, style::TEXT_MUTED).align(Align::Left),
        );
        ui.dl.text(
            &format!("+{value}"),
            Vec2::new(panel.right() - 80.0, y),
            TextStyle::new(FontId::Title, 34.0, style::TEXT_DARK).align(Align::Right),
        );
    }
    let divider_y = panel.y + 370.0;

    ui.dl.image(
        UiTex::Divider,
        Rect::new(panel.x + 70.0, divider_y, panel.w - 140.0, 6.0),
        style::fade(style::TEXT_MUTED, 0.6),
    );

    ui.dl.text(
        "Score",
        Vec2::new(panel.x + 80.0, divider_y + 50.0),
        TextStyle::new(FontId::Title, 44.0, style::TEXT_DARK).align(Align::Left),
    );

    ui.dl.text(
        &format!("{}", stats.total),
        Vec2::new(panel.right() - 80.0, divider_y + 50.0),
        TextStyle::new(FontId::Title, 52.0, style::YELLOW)
            .align(Align::Right)
            .outline(style::INK, 4.0),
    );

    let last = run.level >= LEVEL_COUNT;
    let mut transition = (None, 0);

    if ui
        .button(
            id("next"),
            Rect::centered(
                Vec2::new(screen.x / 2.0, panel.bottom() - 80.0),
                Vec2::new(420.0, 90.0),
            ),
            if last { "La fin..." } else { "Niveau suivant" },
            Some(Icon::Next),
            ButtonStyle::Go,
        )
        .clicked
    {
        if last {
            transition = (
                Some(AppState::cutscene(
                    Scene::Ending,
                    CutsceneNext::Credits(run.score),
                )),
                2,
            );
        } else {
            transition = replace_or_menu(
                AppState::level_start(
                    ctx.settings,
                    SaveState {
                        level: run.level + 1,
                        ..run
                    },
                    true,
                ),
                2,
            );
        }
    }

    ui.prompts(&[(Action::Confirm, "Continuer")]);
    ui.end(ctx.audio);
    transition
}
