use crate::{
    app_state::{AppState, Ctx, Transition},
    graphics::text::FontId,
    settings::save::{GameDifficulty, SaveState},
    ui::{
        UiState,
        draw::TextStyle,
        layout::Rect,
        style,
        textures::Icon,
        ui::{id, idx},
        widgets::{Action, ButtonStyle},
    },
};
use glam::Vec2;

pub fn tick(
    state: &mut UiState,
    ctx: &mut Ctx,
    difficulty: &mut GameDifficulty,
    random_layout: &mut bool,
) -> Transition {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();

    ui.backdrop();
    ui.header("NOUVELLE PARTIE");

    let cards_width = 3.0 * 380.0 + 2.0 * 40.0;
    ui.default_focus(idx("difficulty", 1));

    for (i, ((value, stars, text), rect)) in [
        (
            GameDifficulty::Easy,
            1,
            "Moins d'ennemis et plus lents. Idéal pour les low elo",
        ),
        (
            GameDifficulty::Normal,
            2,
            "L'expérience prévue, pour les normies",
        ),
        (
            GameDifficulty::Hard,
            3,
            "Plus d'ennemis, plus rapides et plus malins. Bonne chance (t'en aura besoin)",
        ),
    ]
    .iter()
    .zip(Rect::new((screen.x - cards_width) / 2.0, 190.0, cards_width, 420.0).columns(3, 40.0))
    .enumerate()
    {
        let selected = *difficulty == *value;
        if ui
            .card(idx("difficulty", i), rect, selected, |ui, r| {
                let top = r.rect.y;

                ui.stars(Vec2::new(r.rect.center().x, top + 90.0), *stars, 3, 72.0);
                ui.dl.text(
                    value.label(),
                    Vec2::new(r.rect.center().x, top + 190.0),
                    TextStyle::new(
                        FontId::Title,
                        54.0,
                        if selected {
                            style::GREEN
                        } else {
                            style::TEXT_DARK
                        },
                    ),
                );
                ui.dl.paragraph(
                    text,
                    Rect::new(r.rect.x + 36.0, top + 240.0, r.rect.w - 72.0, 140.0),
                    TextStyle::new(FontId::Body, 26.0, style::TEXT_MUTED),
                );
            })
            .clicked
        {
            *difficulty = *value;
        }
    }

    if ui.selector(
        id("layout"),
        Rect::new((screen.x - 900.0) / 2.0, 650.0, 900.0, 80.0),
        "Disposition",
        if *random_layout {
            "Aléatoire"
        } else {
            "Classique"
        },
    ) != 0
    {
        *random_layout = !*random_layout;
    }
    ui.dl.text(
        if *random_layout {
            "Caisses, ennemis et sortie sont mélangés à chaque niveau"
        } else {
            "Les niveaux tels qu'ils ont été conçus"
        },
        Vec2::new(screen.x / 2.0, 760.0),
        TextStyle::new(FontId::Body, 26.0, style::WHITE).outline(style::fade(style::INK, 0.7), 2.0),
    );

    let buttons = Rect::new((screen.x - 900.0) / 2.0, 830.0, 900.0, 90.0).columns(2, 40.0);
    let mut transition = (None, 0);
    if ui.back_button(buttons[0]) {
        transition = (None, 1);
    }

    if ui
        .button(
            id("play"),
            buttons[1],
            "Jouer",
            Some(Icon::Next),
            ButtonStyle::Go,
        )
        .clicked
    {
        let run = SaveState::new_game(*difficulty, *random_layout);
        transition = if ctx.settings.unlocked_level > 1 {
            (Some(AppState::stage_map(run)), 0)
        } else {
            match AppState::level_start(ctx.settings, run, true) {
                Some(state) => (Some(state), 1),
                None => (None, 0),
            }
        };
    }

    ui.prompts(&[
        (Action::Navigate, "Déplacer"),
        (Action::Confirm, "Valider"),
        (Action::Back, "Retour"),
    ]);

    ui.end(ctx.audio);
    transition
}
