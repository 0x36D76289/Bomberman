use crate::{
    app_state::Ctx,
    game::game_state::{GameState, HudInfo, player_ui_color},
    graphics::text::FontId,
    ui::{
        UiState,
        draw::{Align, TextStyle},
        layout::Rect,
        style,
        textures::{Icon, UiTex},
        ui::Ui,
        widgets::Action,
    },
};
use glam::{Vec2, Vec4};

type HudPlayer = (u32, bool, bool, Option<&'static str>);

const CARD_HEIGHT: f32 = 84.0;
const TOP: f32 = 20.0;

fn card(ui: &mut Ui, rect: Rect) {
    ui.dl
        .frame(UiTex::Panel, rect, Vec4::ONE.with_w(0.94), 0.45);
}

pub fn draw(state: &mut UiState, game: &GameState, ctx: &mut Ctx) {
    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();

    match game.hud_info() {
        HudInfo::Campaign {
            lives,
            powers,
            pickup,
            level,
            level_count,
            world,
            time_left,
            hurry,
            score,
        } => {
            let lives_rect = Rect::new(style::MARGIN / 2.0, TOP, 190.0, CARD_HEIGHT);
            card(&mut ui, lives_rect);

            ui.dl.image_fit(
                UiTex::Icon(Icon::Singleplayer),
                Rect::centered(
                    Vec2::new(lives_rect.x + 50.0, lives_rect.center().y),
                    Vec2::splat(52.0),
                ),
                style::RED,
            );
            ui.dl.text(
                &format!("x {lives}"),
                Vec2::new(lives_rect.x + 92.0, lives_rect.center().y),
                TextStyle::new(FontId::Title, 42.0, style::TEXT_DARK).align(Align::Left),
            );

            let stage_rect = Rect::new(lives_rect.right() + 16.0, TOP, 330.0, CARD_HEIGHT);
            card(&mut ui, stage_rect);

            ui.dl.text(
                &format!("NIVEAU {level}/{level_count}"),
                Vec2::new(stage_rect.center().x, stage_rect.y + 30.0),
                TextStyle::new(FontId::Title, 32.0, style::TEXT_DARK),
            );
            ui.dl.text(
                world,
                Vec2::new(stage_rect.center().x, stage_rect.y + 62.0),
                TextStyle::new(FontId::Body, 22.0, style::TEXT_MUTED),
            );

            let timer_rect = Rect::centered(
                Vec2::new(screen.x / 2.0, TOP + CARD_HEIGHT / 2.0),
                Vec2::new(250.0, CARD_HEIGHT),
            );
            let urgent = hurry || time_left < 30.0;

            ui.dl.frame(
                UiTex::Panel,
                timer_rect,
                if urgent {
                    Vec4::new(1.0, 0.75, 0.72, 1.0)
                } else {
                    Vec4::ONE
                },
                0.45,
            );
            let seconds = time_left.ceil() as u32;

            ui.dl.text(
                &if hurry {
                    "DEPECHE-TOI !".to_string()
                } else {
                    format!("{}:{:02}", seconds / 60, seconds % 60)
                },
                timer_rect.center(),
                TextStyle::new(
                    FontId::Title,
                    46.0,
                    if urgent { style::RED } else { style::TEXT_DARK },
                ),
            );

            let stats = format!(
                "BOMBES {}   FLAMME {}   PATINS {}",
                powers.bombs, powers.fire, powers.speed
            );
            let specials: Vec<&str> = [
                (powers.kick, "Coup de pied"),
                (powers.remote, "Télécommande"),
                (powers.wall_pass, "Passe-murs"),
                (powers.bomb_pass, "Passe-bombes"),
            ]
            .into_iter()
            .filter_map(|(on, name)| on.then_some(name))
            .collect();
            let stats_style =
                TextStyle::new(FontId::Title, 24.0, style::TEXT_DARK).align(Align::Left);
            let specials_style =
                TextStyle::new(FontId::Body, 20.0, style::TEXT_MUTED).align(Align::Left);
            let specials_text = specials.join(" · ");
            let height = if specials.is_empty() { 56.0 } else { 84.0 };
            let powers_rect = Rect::new(
                style::MARGIN / 2.0,
                screen.y - TOP - height,
                ui.dl
                    .text_width(&stats, &stats_style)
                    .max(ui.dl.text_width(&specials_text, &specials_style))
                    + 48.0,
                height,
            );
            card(&mut ui, powers_rect);
            ui.dl.text(
                &stats,
                Vec2::new(powers_rect.x + 24.0, powers_rect.y + 28.0),
                stats_style,
            );
            if !specials.is_empty() {
                ui.dl.text(
                    &specials_text,
                    Vec2::new(powers_rect.x + 24.0, powers_rect.y + 60.0),
                    specials_style,
                );
            }

            if let Some(name) = pickup {
                ui.dl.text(
                    name,
                    Vec2::new(screen.x / 2.0, TOP + CARD_HEIGHT + 36.0),
                    TextStyle::new(FontId::Title, 40.0, style::YELLOW).outline(style::INK, 4.0),
                );
            }

            let score_rect = Rect::new(
                screen.x - style::MARGIN / 2.0 - 320.0,
                TOP,
                320.0,
                CARD_HEIGHT,
            );
            card(&mut ui, score_rect);

            ui.dl.text(
                "SCORE",
                Vec2::new(score_rect.x + 26.0, score_rect.center().y),
                TextStyle::new(FontId::Title, 26.0, style::TEXT_MUTED).align(Align::Left),
            );
            ui.dl.text(
                &format!("{score}"),
                Vec2::new(score_rect.right() - 26.0, score_rect.center().y),
                TextStyle::new(FontId::Title, 44.0, style::TEXT_DARK).align(Align::Right),
            );
        }

        HudInfo::Multiplayer(players, teams) => {
            let mut groups: Vec<Vec<HudPlayer>> = Vec::new();
            match teams.as_deref() {
                Some(t) => {
                    let mut order: Vec<u32> = t.to_vec();
                    order.sort_unstable();
                    order.dedup();
                    for team in order {
                        groups.push(
                            players
                                .iter()
                                .filter(|p| t.get(p.0 as usize) == Some(&team))
                                .copied()
                                .collect(),
                        );
                    }
                }
                None => groups = players.iter().map(|p| vec![*p]).collect(),
            }

            let gap = 14.0;
            let slots: usize = groups.iter().map(Vec::len).sum();
            let width = ((screen.x - style::MARGIN - gap * (groups.len() as f32 - 1.0))
                / slots.max(1) as f32)
                .min(210.0);
            let mut x =
                (screen.x - (slots as f32 * width + gap * (groups.len() as f32 - 1.0))) / 2.0;

            for group in groups {
                let panel = Rect::new(x, TOP, width * group.len() as f32, 70.0);
                ui.dl.frame(
                    UiTex::Panel,
                    panel,
                    Vec4::new(
                        1.0,
                        1.0,
                        1.0,
                        0.94 * if group.iter().any(|p| p.2) { 1.0 } else { 0.45 },
                    ),
                    0.4,
                );
                for (j, (player, human, alive, curse)) in group.into_iter().enumerate() {
                    player_entry(
                        &mut ui,
                        Rect::new(panel.x + j as f32 * width, TOP, width, 70.0),
                        player,
                        human,
                        alive,
                        curse,
                        player_ui_color(player as usize, teams.as_deref()),
                    );
                }
                x += panel.w + gap;
            }
        }
    }

    let banner = game.banner();
    if let Some(banner) = &banner {
        let center = Vec2::new(screen.x / 2.0, screen.y * 0.45);

        ui.dl.rect(
            Rect::centered(center, Vec2::new(screen.x + 200.0, 230.0)),
            Vec4::new(0.08, 0.04, 0.18, 0.75),
        );
        ui.dl.rect(
            Rect::centered(
                center - Vec2::new(0.0, 115.0),
                Vec2::new(screen.x + 200.0, 8.0),
            ),
            style::YELLOW,
        );
        ui.dl.rect(
            Rect::centered(
                center + Vec2::new(0.0, 115.0),
                Vec2::new(screen.x + 200.0, 8.0),
            ),
            style::YELLOW,
        );
        ui.dl.text(
            &banner.title,
            center - Vec2::new(0.0, 30.0),
            TextStyle::new(FontId::Title, 112.0, style::YELLOW)
                .outline(style::INK, 7.0)
                .shadow(style::fade(style::INK, 0.6), Vec2::new(0.0, 10.0)),
        );
        ui.dl.text(
            &banner.subtitle,
            center + Vec2::new(0.0, 68.0),
            TextStyle::new(FontId::Title, 38.0, style::WHITE).outline(style::INK, 3.0),
        );
    }
    if banner.is_none() && ui.time() < 10.0 {
        ui.prompts(&[(Action::Pause, "Pause")]);
    }
    ui.end(ctx.audio);
}

fn player_entry(
    ui: &mut Ui,
    rect: Rect,
    player: u32,
    human: bool,
    alive: bool,
    curse: Option<&'static str>,
    color: Vec4,
) {
    let alpha = if alive { 1.0 } else { 0.45 };

    ui.dl.image_fit(
        UiTex::Icon(Icon::Singleplayer),
        Rect::centered(Vec2::new(rect.x + 38.0, rect.center().y), Vec2::splat(40.0)),
        style::fade(color, alpha),
    );

    ui.dl.text(
        &if human {
            format!("P{}", player + 1)
        } else {
            format!("IA {}", player + 1)
        },
        Vec2::new(rect.x + 70.0, rect.center().y),
        TextStyle::new(FontId::Title, 30.0, style::fade(style::TEXT_DARK, alpha))
            .align(Align::Left),
    );

    if !alive {
        ui.dl.image_fit(
            UiTex::Icon(Icon::Cross),
            Rect::centered(
                Vec2::new(rect.right() - 34.0, rect.center().y),
                Vec2::splat(40.0),
            ),
            style::RED,
        );
    }
    if let Some(curse) = curse {
        ui.dl.text(
            curse,
            Vec2::new(rect.center().x, rect.bottom() + 20.0),
            TextStyle::new(FontId::Title, 24.0, style::RED).outline(style::INK, 3.0),
        );
    }
}
