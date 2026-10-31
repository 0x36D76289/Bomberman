use crate::{
    app_state::{AppState, Ctx, Transition},
    game::game_state::{RestartInfo, player_ui_color},
    graphics::text::FontId,
    ui::{
        UiState,
        draw::TextStyle,
        layout::Rect,
        style,
        textures::{Icon, UiTex},
        ui::id,
        widgets::{Action, ButtonStyle},
    },
};
use glam::Vec2;

pub fn tick(
    state: &mut UiState,
    ctx: &mut Ctx,
    winners: &[u32],
    restart: &RestartInfo,
) -> Transition {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();

    ui.fill(style::OVERLAY);

    let (humans, teams) = match restart {
        RestartInfo::Multiplayer { settings, .. } => {
            (settings.nb_humans, settings.teams.as_deref())
        }
        RestartInfo::Campaign(_) => (1, None),
    };
    let name = |id: u32| {
        if id < humans {
            format!("Joueur {}", id + 1)
        } else {
            format!("IA {}", id + 1)
        }
    };

    ui.title(
        &match winners {
            [] => "ÉGALITÉ !".to_string(),
            [first, ..]
                if let Some(t) = teams
                    && winners
                        .iter()
                        .all(|w| t.get(*w as usize) == t.get(*first as usize)) =>
            {
                format!(
                    "ÉQUIPE {} GAGNE !",
                    t.get(*first as usize).map_or(0, |team| team + 1)
                )
            }
            [winner] => format!("{} GAGNE !", name(*winner).to_uppercase()),
            _ => "ÉGALITÉ !".to_string(),
        },
        Vec2::new(screen.x / 2.0, 250.0),
        110.0,
    );

    ui.dl.image_fit(
        UiTex::Icon(Icon::Trophy),
        Rect::centered(Vec2::new(screen.x / 2.0, 450.0), Vec2::splat(200.0)),
        style::YELLOW,
    );

    for (i, winner) in winners.iter().enumerate() {
        let chip = Rect::centered(
            Vec2::new(
                screen.x / 2.0 - winners.len().max(1) as f32 * 260.0 / 2.0
                    + 130.0
                    + i as f32 * 260.0,
                620.0,
            ),
            Vec2::new(240.0, 70.0),
        );

        ui.dl.frame(UiTex::Panel, chip, glam::Vec4::ONE, 0.4);
        ui.dl.image_fit(
            UiTex::Icon(Icon::Singleplayer),
            Rect::new(chip.x + 16.0, chip.y + 12.0, 46.0, 46.0),
            player_ui_color(*winner as usize, teams),
        );
        ui.dl.text(
            &name(*winner),
            chip.center() + Vec2::new(24.0, 0.0),
            TextStyle::new(FontId::Title, 32.0, style::TEXT_DARK),
        );
    }

    let buttons = Rect::new((screen.x - 860.0) / 2.0, 740.0, 860.0, 90.0).columns(2, 40.0);
    let mut transition = (None, 0);

    ui.default_focus(id("again"));
    if ui
        .button(
            id("lobby"),
            buttons[0],
            "Salon",
            Some(Icon::Return),
            ButtonStyle::Normal,
        )
        .clicked
        || ui.back_pressed()
    {
        transition = (None, 2);
    }

    if ui
        .button(
            id("again"),
            buttons[1],
            "Revanche",
            Some(Icon::Next),
            ButtonStyle::Go,
        )
        .clicked
        && let Some(game) = restart.restart(ctx.resources)
    {
        transition = (Some(AppState::game(game)), 2);
    }

    ui.prompts(&[(Action::Navigate, "Déplacer"), (Action::Confirm, "Valider")]);
    ui.end(ctx.audio);
    transition
}
