use crate::{
    app_state::{AppState, Ctx, Transition},
    game::{
        game_settings::GameSettings,
        game_state::{GameState, player_ui_color, team_color},
        lobby::{LobbyPreset, LobbySettings, Slot, TEAM_COUNT, TEAM_SIZE},
        map::map::Map,
    },
    graphics::{StateRenderInfo, text::FontId},
    input::controller::uses_controller,
    ui::{
        UiState,
        draw::{Align, TextStyle},
        layout::Rect,
        style,
        textures::{Icon, UiTex},
        ui::{Ui, id, idx},
        widgets::{Action, ButtonStyle},
    },
};
use glam::{Vec2, Vec4};

fn to_ndc(rect: Rect, screen: Vec2) -> StateRenderInfo {
    let corner = |p: Vec2| [p.x / screen.x * 2.0 - 1.0, p.y / screen.y * 2.0 - 1.0];
    StateRenderInfo {
        top_left_coord: corner(Vec2::new(rect.x, rect.y)),
        bottom_right_coord: corner(Vec2::new(rect.right(), rect.bottom())),
        drawn_first: false,
    }
}

pub fn tick(
    state: &mut UiState,
    ctx: &mut Ctx,
    lobby: &mut LobbySettings,
    mut preview: Option<&mut GameState>,
) -> Transition {
    let humans = (ctx.inputs.len() as u8).clamp(1, lobby.preset.max_players());
    if humans != lobby.humans {
        lobby.set_humans(humans);
    }

    let mut ui = super::begin(state, ctx);
    let screen = ui.screen();

    ui.backdrop();
    ui.header("MULTIJOUEUR");

    let preview_height = 560.0;
    let preview_rect = Rect::new(
        style::MARGIN,
        190.0,
        preview_height * screen.x / screen.y,
        preview_height,
    );
    let frame = preview_rect.inset(-14.0);
    ui.dl.frame(UiTex::PanelGrey, frame, Vec4::ONE, 0.6);

    if let Some(preview) = preview.as_mut() {
        preview.render_info = to_ndc(preview_rect, screen);
    }

    ui.dl.text(
        lobby.preset.description(),
        Vec2::new(preview_rect.center().x, preview_rect.bottom() + 50.0),
        TextStyle::new(FontId::Body, 28.0, style::WHITE).outline(style::fade(style::INK, 0.7), 2.0),
    );

    let column = Rect::new(
        frame.right() + 50.0,
        190.0,
        screen.x - frame.right() - 50.0 - style::MARGIN,
        700.0,
    );
    let presets = Rect::new(column.x, column.y, column.w, 76.0).columns(4, 12.0);

    for (i, preset) in [
        LobbyPreset::Corners,
        LobbyPreset::Arena,
        LobbyPreset::Teams,
        LobbyPreset::Custom,
    ]
    .into_iter()
    .enumerate()
    {
        if ui
            .button(
                idx("preset", i),
                presets[i],
                preset.name(),
                None,
                if lobby.preset == preset {
                    ButtonStyle::Go
                } else {
                    ButtonStyle::Normal
                },
            )
            .clicked
        {
            lobby.set_preset(preset);
        }
    }
    let rows = Rect::new(column.x, column.y + 100.0, column.w, 6.0 * 56.0 + 5.0 * 8.0).rows(6, 8.0);

    lobby.step_width(ui.selector(id("width"), rows[0], "Largeur", &lobby.width.to_string()));

    lobby.step_height(ui.selector(id("height"), rows[1], "Hauteur", &lobby.height.to_string()));

    let mut open = lobby.cheesiness as f32 / 100.0;

    if ui.slider(id("open"), rows[2], "Espace libre", &mut open) {
        lobby.cheesiness = (open * 100.0).round() as u8;
    }

    lobby.step_bots(ui.selector(id("bots"), rows[3], "Joueurs IA", &lobby.bots.to_string()));

    lobby.step_bot_level(ui.selector(
        id("bot_level"),
        rows[4],
        "Niveau des IA",
        lobby.bot_level.name(),
    ));

    lobby.step_theme(ui.selector(id("theme"), rows[5], "Thème", lobby.theme.short_name()));

    let players_top = rows[5].bottom() + 24.0;
    let chip_count = if lobby.is_teams() {
        TEAM_COUNT * TEAM_SIZE
    } else {
        lobby.players() as usize
    };
    let chip_columns = if chip_count > 8 { 6 } else { 4 };
    let chip_rows = chip_count.div_ceil(chip_columns).max(1);
    let chip_gap = 10.0;
    let chip_size = Vec2::new(
        (column.w - (chip_columns - 1) as f32 * chip_gap) / chip_columns as f32,
        ((screen.y - 200.0 - 16.0 - (players_top + 50.0) - (chip_rows - 1) as f32 * chip_gap)
            / chip_rows as f32)
            .clamp(24.0, 56.0),
    );
    let chip_rect = |i: usize| {
        Rect::new(
            column.x + (i % chip_columns) as f32 * (chip_size.x + chip_gap),
            players_top + 50.0 + (i / chip_columns) as f32 * (chip_size.y + chip_gap),
            chip_size.x,
            chip_size.y,
        )
    };

    if lobby.is_teams() {
        team_slots(&mut ui, ctx, lobby, players_top, chip_rect);
    } else {
        ui.dl.text(
            &format!("Joueurs {}/{}", lobby.players(), lobby.preset.max_players()),
            Vec2::new(column.x, players_top + 20.0),
            TextStyle::new(FontId::Title, 32.0, style::WHITE)
                .align(Align::Left)
                .outline(style::INK, 3.0),
        );
        for i in 0..lobby.players() as usize {
            let chip = chip_rect(i);
            ui.dl.frame(UiTex::Panel, chip, Vec4::ONE, 0.35);
            let (slot, color) = if i < lobby.humans as usize {
                (Slot::Human(i as u8), player_ui_color(i, None))
            } else {
                (Slot::Bot, style::TEXT_MUTED)
            };
            slot_content(&mut ui, ctx, chip, slot, color);
        }
    }

    let buttons = Rect::new(column.x, screen.y - 200.0, column.w, 90.0).columns(2, 30.0);
    let mut transition = (None, 0);

    if ui.back_button(buttons[0]) {
        transition = (None, 1);
    }

    ui.default_focus(id("start"));
    if ui
        .button(
            id("start"),
            buttons[1],
            "Lancer",
            Some(Icon::Next),
            if lobby.players() >= 2 {
                ButtonStyle::Go
            } else {
                ButtonStyle::Disabled
            },
        )
        .clicked
    {
        transition = (
            Some(AppState::lobby_start(
                ctx,
                lobby,
                preview.as_ref().map(|p| p.get_map().clone()),
            )),
            0,
        );
    }
    ui.prompts(&[
        (Action::Navigate, "Déplacer"),
        (Action::Change, "Modifier"),
        (Action::Back, "Retour"),
    ]);

    ui.end(ctx.audio);
    transition
}

fn team_slots(
    ui: &mut Ui,
    ctx: &Ctx,
    lobby: &mut LobbySettings,
    players_top: f32,
    chip_rect: impl Fn(usize) -> Rect,
) {
    for team in 0..TEAM_COUNT {
        ui.dl.text(
            &format!("Équipe {}", team + 1),
            Vec2::new(chip_rect(team).center().x, players_top + 20.0),
            TextStyle::new(FontId::Title, 28.0, team_color(team).extend(1.0))
                .outline(style::INK, 3.0),
        );
    }
    for index in 0..TEAM_COUNT * TEAM_SIZE {
        let team = index / TEAM_SIZE;
        let chip = chip_rect(index % TEAM_SIZE * TEAM_COUNT + team);
        if ui
            .card(idx("slot", index), chip, false, |ui, _| {
                slot_content(
                    ui,
                    ctx,
                    chip,
                    lobby.slots[index],
                    team_color(team).extend(1.0),
                );
            })
            .clicked
        {
            lobby.cycle_slot(index);
        }
    }
}

fn slot_content(ui: &mut Ui, ctx: &Ctx, chip: Rect, slot: Slot, color: Vec4) {
    let icon_size = (chip.h * 0.68).min(38.0);
    let icon_rect = Rect::centered(
        Vec2::new(chip.x + 13.0 + icon_size / 2.0, chip.center().y),
        Vec2::splat(icon_size),
    );
    let (name, text_color) = match slot {
        Slot::Human(k) => {
            ui.dl.image_fit(
                UiTex::Icon(
                    if ctx
                        .settings
                        .binds
                        .get(k as usize)
                        .is_some_and(uses_controller)
                    {
                        Icon::Gamepad
                    } else {
                        Icon::Singleplayer
                    },
                ),
                icon_rect,
                color,
            );
            (format!("J{}", k + 1), style::TEXT_DARK)
        }
        Slot::Bot => {
            ui.dl.image_fit(
                UiTex::Icon(Icon::Singleplayer),
                icon_rect,
                style::fade(color, 0.8),
            );
            ("IA".to_string(), style::TEXT_DARK)
        }
        Slot::Empty => ("+ IA".to_string(), style::TEXT_MUTED),
    };
    ui.dl.text(
        &name,
        Vec2::new(icon_rect.right() + 10.0, chip.center().y),
        TextStyle::new(FontId::Title, (chip.h * 0.5).min(28.0), text_color).align(Align::Left),
    );
}

impl AppState {
    fn lobby_start(ctx: &Ctx, lobby: &LobbySettings, map: Option<Map>) -> AppState {
        AppState::game(GameState::new_multiplayer_from_map(
            ctx.resources,
            GameSettings {
                nb_humans: lobby.humans as u32,
                nb_bots: lobby.bots as u32,
                bot_level: lobby.bot_level,
                teams: lobby.player_teams(),
            },
            map.or_else(|| Map::new(lobby.map_settings(), ctx.resources))
                .unwrap_or_else(|| Map::backdrop(ctx.resources, lobby.theme)),
        ))
    }
}
