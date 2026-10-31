use crate::{
    app_state::{AppState, Ctx, POP_TO_MENU, Transition, replace_or_menu},
    game::game_state::RestartInfo,
    ui::{
        UiState,
        layout::Rect,
        style,
        textures::Icon,
        ui::id,
        widgets::{Action, ButtonStyle},
    },
};
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseConfirm {
    Restart,
    Quit,
}

fn restart_game(ctx: &Ctx, restart: &RestartInfo) -> Transition {
    replace_or_menu(restart.restart(ctx.resources).map(AppState::game), 2)
}

pub fn tick(
    state: &mut UiState,
    ctx: &mut Ctx,
    restart: &RestartInfo,
    confirm: &mut Option<PauseConfirm>,
) -> Transition {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();
    let campaign = matches!(restart, RestartInfo::Campaign(_));
    ui.fill(style::OVERLAY);
    let mut transition = (None, 0);

    match *confirm {
        None => {
            let panel = Rect::centered(screen / 2.0, Vec2::new(620.0, 600.0));
            ui.panel(panel);
            ui.title("PAUSE", Vec2::new(screen.x / 2.0, panel.y + 80.0), 80.0);
            let rows = Rect::new(
                panel.x + 60.0,
                panel.y + 160.0,
                panel.w - 120.0,
                4.0 * 84.0 + 3.0 * 18.0,
            )
            .rows(4, 18.0);
            if ui
                .button(
                    id("resume"),
                    rows[0],
                    "Reprendre",
                    Some(Icon::Next),
                    ButtonStyle::Go,
                )
                .clicked
                || ui.back_pressed()
            {
                ctx.audio.resume_background_music();
                transition = (None, 1);
            }
            if ui
                .button(
                    id("restart"),
                    rows[1],
                    "Recommencer",
                    Some(Icon::Return),
                    ButtonStyle::Normal,
                )
                .clicked
            {
                *confirm = Some(PauseConfirm::Restart);
            }
            if ui
                .button(
                    id("settings"),
                    rows[2],
                    "Paramètres",
                    Some(Icon::Gear),
                    ButtonStyle::Normal,
                )
                .clicked
            {
                transition = (Some(AppState::settings(true)), 0);
            }
            if ui
                .button(
                    id("quit"),
                    rows[3],
                    if campaign {
                        "Menu principal"
                    } else {
                        "Retour au salon"
                    },
                    Some(Icon::Home),
                    ButtonStyle::Danger,
                )
                .clicked
            {
                *confirm = Some(PauseConfirm::Quit);
            }
            ui.prompts(&[
                (Action::Navigate, "Déplacer"),
                (Action::Confirm, "Valider"),
                (Action::Back, "Reprendre"),
            ]);
        }
        Some(PauseConfirm::Restart) => {
            match ui.dialog(
                "restart",
                "Recommencer ?",
                if campaign {
                    "Le niveau recommence avec le même nombre de vies."
                } else {
                    "La manche recommence sur la même carte."
                },
                "Recommencer",
            ) {
                Some(true) => {
                    ctx.audio.resume_background_music();
                    transition = restart_game(ctx, restart);
                }
                Some(false) => *confirm = None,
                None => (),
            }
        }
        Some(PauseConfirm::Quit) => {
            match ui.dialog(
                "quit",
                "Quitter la partie ?",
                if campaign {
                    "Tu reprendras au début de ce niveau."
                } else {
                    "La manche sera perdue."
                },
                "Quitter",
            ) {
                Some(true) => {
                    ctx.audio.resume_background_music();
                    transition = if campaign {
                        (None, POP_TO_MENU)
                    } else {
                        (None, 2)
                    };
                }
                Some(false) => *confirm = None,
                None => (),
            }
        }
    }
    ui.end(ctx.audio);
    transition
}
