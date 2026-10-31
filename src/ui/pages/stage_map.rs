use crate::{
    app_state::{AppState, Ctx, Transition, replace_or_menu},
    game::map::map::{LEVEL_COUNT, Map},
    graphics::text::FontId,
    settings::save::SaveState,
    ui::{
        UiState,
        draw::TextStyle,
        layout::Rect,
        style,
        textures::{Icon, UiTex},
        ui::idx,
        widgets::Action,
    },
};
use glam::{Vec2, Vec4};

pub fn tick(state: &mut UiState, ctx: &mut Ctx, run: SaveState) -> Transition {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();
    let unlocked = ctx.settings.unlocked_level.clamp(1, LEVEL_COUNT);

    ui.backdrop();
    ui.header("CHOISIS UN NIVEAU");
    ui.dl.text(
        &format!(
            "{} - disposition {}",
            run.difficulty.label(),
            if run.random_layout {
                "aléatoire"
            } else {
                "classique"
            }
        ),
        Vec2::new(screen.x / 2.0, 160.0),
        TextStyle::new(FontId::Title, 28.0, style::WHITE).outline(style::INK, 3.0),
    );

    let (columns, gap) = (LEVEL_COUNT as usize, 24.0);
    let card = Vec2::new(
        ((screen.x - 2.0 * style::MARGIN - (columns - 1) as f32 * gap) / columns as f32).min(340.0),
        250.0,
    );
    let mut transition = (None, 0);

    ui.default_focus(idx("stage", unlocked as usize - 1));

    for level in 1..=LEVEL_COUNT {
        let i = level as usize - 1;
        let pos = Vec2::new(
            (screen.x - (columns as f32 * card.x + (columns - 1) as f32 * gap)) / 2.0,
            220.0,
        ) + Vec2::new(
            (i % columns) as f32 * (card.x + gap),
            (i / columns) as f32 * (card.y + gap),
        );
        let (name, theme) = Map::level_info(level).unwrap_or_default();
        let locked = level > unlocked;

        if ui
            .card(
                idx("stage", i),
                Rect::new(pos.x, pos.y, card.x, card.y),
                false,
                |ui, r| {
                    let rect = r.rect;
                    let band = Rect::new(rect.x + 14.0, rect.y + 14.0, rect.w - 28.0, 92.0);

                    ui.dl.frame(
                        UiTex::PanelGrey,
                        band,
                        if locked {
                            Vec4::new(0.6, 0.6, 0.65, 1.0)
                        } else {
                            theme.ui_color()
                        },
                        0.4,
                    );
                    ui.dl.text(
                        &format!("{level}"),
                        band.center(),
                        TextStyle::new(FontId::Title, 64.0, style::WHITE).outline(style::INK, 4.0),
                    );

                    ui.dl.text(
                        theme.display_name(),
                        Vec2::new(rect.center().x, rect.y + 145.0),
                        TextStyle::new(
                            FontId::Title,
                            26.0,
                            if locked {
                                style::TEXT_MUTED
                            } else {
                                style::TEXT_DARK
                            },
                        ),
                    );
                    ui.dl.text(
                        if locked { "Verrouillé" } else { &name },
                        Vec2::new(rect.center().x, rect.y + 190.0),
                        TextStyle::new(FontId::Body, 24.0, style::TEXT_MUTED),
                    );

                    let badge = Rect::centered(
                        Vec2::new(rect.right() - 34.0, rect.y + 34.0),
                        Vec2::splat(56.0),
                    );
                    if locked {
                        ui.dl
                            .image_fit(UiTex::Icon(Icon::Locked), badge, style::INK);
                    } else if level < unlocked {
                        ui.dl.image_fit(UiTex::Star, badge, Vec4::ONE);
                    }
                },
            )
            .clicked
        {
            if locked {
                ui.sound(crate::audio::SoundEffect::NoBomb);
            } else {
                transition = replace_or_menu(
                    AppState::level_start(ctx.settings, SaveState { level, ..run }, true),
                    2,
                );
            }
        }
    }

    if ui.back_button(Rect::new(style::MARGIN, screen.y - 140.0, 320.0, 84.0)) {
        transition = (None, 1);
    }
    ui.prompts(&[
        (Action::Navigate, "Déplacer"),
        (Action::Confirm, "Jouer"),
        (Action::Back, "Retour"),
    ]);
    ui.end(ctx.audio);
    transition
}
