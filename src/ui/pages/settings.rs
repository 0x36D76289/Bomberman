use crate::{
    app_state::{Ctx, Transition},
    audio::SoundEffect,
    graphics::text::FontId,
    input::{
        controller::{create_bind, is_bindable_action, uses_controller},
        event::UNBOUND_NAME,
        input::{BIND_LEN, default_binds},
    },
    settings::settings::Settings,
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

const BIND_NAMES: [&str; BIND_LEN] = [
    "Haut",
    "Bas",
    "Gauche",
    "Droite",
    "Bombe / Valider",
    "Pause / Retour",
    "Détoner",
];
const BIND_TIMEOUT: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Video,
    Audio,
    Controls,
}

#[derive(Debug, Clone)]
pub struct SettingsPage {
    pub in_game: bool,
    tab: Tab,
    player: usize,
    waiting: Option<(usize, f32)>,
    confirm_remove: bool,
}

fn step_value<T: Copy + PartialEq>(list: &[T], current: T, direction: i32) -> T {
    list[(list.iter().position(|v| *v == current).unwrap_or(0) as i32 + direction)
        .clamp(0, list.len() as i32 - 1) as usize]
}

impl SettingsPage {
    pub fn new(in_game: bool) -> Self {
        Self {
            in_game,
            tab: Tab::Video,
            player: 0,
            waiting: None,
            confirm_remove: false,
        }
    }

    pub fn tick(&mut self, state: &mut UiState, ctx: &mut Ctx) -> Transition {
        let mut ui = super::begin(state, ctx);
        let screen = ui.screen();
        if self.in_game {
            ui.fill(style::OVERLAY);
        } else {
            ui.backdrop();
        }
        ui.header("PARAMÈTRES");

        let content = Rect::new((screen.x - 1100.0) / 2.0, 250.0, 1100.0, 600.0);
        let mut transition = (None, 0);

        if !(self.waiting.is_some() || self.confirm_remove) {
            let tabs = Rect::new(content.x, 150.0, content.w, 80.0).columns(3, 24.0);
            for (i, (tab, name, icon)) in [
                (Tab::Video, "Vidéo", Icon::Gear),
                (Tab::Audio, "Audio", Icon::Music),
                (Tab::Controls, "Commandes", Icon::Gamepad),
            ]
            .into_iter()
            .enumerate()
            {
                if ui
                    .button(
                        idx("tab", i),
                        tabs[i],
                        name,
                        Some(icon),
                        if self.tab == tab {
                            ButtonStyle::Go
                        } else {
                            ButtonStyle::Normal
                        },
                    )
                    .clicked
                {
                    self.tab = tab;
                }
            }

            if match self.tab {
                Tab::Video => video(&mut ui, ctx.settings, content),
                Tab::Audio => audio(&mut ui, ctx.settings, content),
                Tab::Controls => self.controls(&mut ui, ctx, content),
            } {
                ctx.settings.save();
                ctx.audio.set_volume(ctx.settings);
            }

            if ui.back_button(Rect::new(content.x, screen.y - 150.0, 340.0, 84.0)) {
                transition = (None, 1);
            }
            ui.prompts(&[
                (Action::Navigate, "Déplacer"),
                (Action::Change, "Modifier"),
                (Action::Back, "Retour"),
            ]);
        } else if let Some((slot, time)) = &mut self.waiting {
            *time += ctx.delta;

            let rect = Rect::centered(screen / 2.0, Vec2::new(860.0, 340.0));

            ui.fill(style::OVERLAY);
            ui.dl.frame(UiTex::Panel, rect, Vec4::ONE, 0.6);
            ui.dl.text(
                &format!("Joueur {} - {}", self.player + 1, BIND_NAMES[*slot]),
                Vec2::new(rect.center().x, rect.y + 80.0),
                TextStyle::new(FontId::Title, 44.0, style::TEXT_DARK),
            );

            ui.dl.text(
                "Appuie sur une touche ou un bouton...",
                Vec2::new(rect.center().x, rect.y + 170.0),
                TextStyle::new(FontId::Title, 40.0, style::YELLOW).outline(style::INK, 3.0),
            );
            ui.dl.text(
                &format!(
                    "Annulation dans {:.0} s",
                    (BIND_TIMEOUT - *time).max(0.0).ceil()
                ),
                Vec2::new(rect.center().x, rect.y + 260.0),
                TextStyle::new(FontId::Body, 24.0, style::TEXT_MUTED),
            );

            if let Some(event) = ctx.events.iter().find(|e| is_bindable_action(e)) {
                if let Some(binds) = ctx.settings.binds.get_mut(self.player) {
                    binds[*slot] = create_bind(event);
                    ctx.settings.save();
                }
                ui.sound(SoundEffect::MenuConfirm);
                self.waiting = None;
            } else if *time > BIND_TIMEOUT || ui.input.mouse_pressed {
                ui.sound(SoundEffect::MenuBack);
                self.waiting = None;
            }
        } else if self.confirm_remove {
            match ui.dialog(
                "remove",
                &format!("Retirer le joueur {} ?", self.player + 1),
                "Ses touches seront supprimées.",
                "Retirer",
            ) {
                Some(true) => {
                    if ctx.settings.binds.len() > 1 && self.player < ctx.settings.binds.len() {
                        ctx.settings.binds.remove(self.player);
                        ctx.settings.save();
                    }
                    self.player = self.player.min(ctx.settings.binds.len() - 1);
                    self.confirm_remove = false;
                }
                Some(false) => self.confirm_remove = false,
                None => (),
            }
        }
        ui.end(ctx.audio);
        transition
    }

    fn controls(&mut self, ui: &mut Ui, ctx: &mut Ctx, content: Rect) -> bool {
        let settings = &mut *ctx.settings;
        let count = settings.binds.len();

        self.player = self.player.min(count.saturating_sub(1));

        let (top, rest) = content.take_top(84.0);
        let step = ui.selector(
            id("player"),
            top,
            "Joueur",
            &format!(
                "Joueur {}{}",
                self.player + 1,
                if uses_controller(&settings.binds[self.player]) {
                    " (+ manette)"
                } else {
                    ""
                }
            ),
        );
        if step != 0 {
            self.player = (self.player as i32 + step).rem_euclid(count as i32) as usize;
        }

        let (binds_area, actions) = rest.take_top(8.0 * 52.0 + 20.0);
        let rows = binds_area
            .inset(10.0)
            .translate(Vec2::new(0.0, 10.0))
            .rows(4, 12.0);
        for (i, name) in BIND_NAMES.iter().enumerate() {
            let key = settings.binds[self.player][i].to_string();

            if ui
                .card(
                    idx("bind", i),
                    rows[i % 4].columns(2, 24.0)[i / 4],
                    false,
                    |ui, r| {
                        let rect = r.rect;
                        ui.dl.text(
                            name,
                            Vec2::new(rect.x + 24.0, rect.center().y),
                            TextStyle::new(FontId::Title, 26.0, style::TEXT_DARK)
                                .align(Align::Left),
                        );
                        let key_rect =
                            Rect::new(rect.right() - 220.0, rect.y + 10.0, 200.0, rect.h - 20.0);
                        ui.dl.frame(UiTex::PanelGrey, key_rect, Vec4::ONE, 0.35);

                        let color = if key == UNBOUND_NAME {
                            style::RED
                        } else {
                            style::TEXT_DARK
                        };
                        ui.dl.text(
                            &key,
                            key_rect.center(),
                            TextStyle::new(
                                FontId::Body,
                                24.0 * ((key_rect.w - 20.0)
                                    / ui.dl
                                        .text_width(
                                            &key,
                                            &TextStyle::new(FontId::Body, 24.0, color),
                                        )
                                        .max(1.0))
                                .min(1.0),
                                color,
                            ),
                        );
                    },
                )
                .clicked
            {
                self.waiting = Some((i, 0.0));
            }
        }

        let buttons = Rect::new(actions.x, actions.y + 24.0, actions.w, 76.0).columns(3, 24.0);
        let mut changed = false;
        if ui
            .button(
                id("add"),
                buttons[0],
                "Ajouter",
                Some(Icon::Plus),
                if count >= 8 {
                    ButtonStyle::Disabled
                } else {
                    ButtonStyle::Normal
                },
            )
            .clicked
        {
            settings.binds.push(default_binds());
            self.player = settings.binds.len() - 1;
            changed = true;
        }
        if ui
            .button(
                id("reset"),
                buttons[1],
                "Par défaut",
                Some(Icon::Return),
                ButtonStyle::Normal,
            )
            .clicked
        {
            settings.binds[self.player] = Settings::player_binds(self.player);
            changed = true;
        }
        if ui
            .button(
                id("remove"),
                buttons[2],
                "Retirer",
                Some(Icon::Trashcan),
                if count <= 1 {
                    ButtonStyle::Disabled
                } else {
                    ButtonStyle::Danger
                },
            )
            .clicked
        {
            self.confirm_remove = true;
        }
        changed
    }
}

fn video(ui: &mut Ui, settings: &mut Settings, content: Rect) -> bool {
    let rows = Rect {
        h: 5.0 * 84.0 + 4.0 * 18.0,
        ..content
    }
    .rows(5, 18.0);

    let mut changed = false;
    let step = ui.selector(
        id("resolution"),
        rows[0],
        "Taille de la fenêtre",
        &format!("{} x {}", settings.resolution.0, settings.resolution.1),
    );
    if step != 0 {
        settings.resolution = step_value(
            &[
                (800, 600),
                (1024, 768),
                (1280, 720),
                (1280, 800),
                (1366, 768),
                (1600, 900),
                (1680, 1050),
                (1920, 1080),
                (2560, 1440),
                (3840, 2160),
            ],
            settings.resolution,
            step,
        );
        changed = true;
    }

    changed |= ui.toggle(
        id("fullscreen"),
        rows[1],
        "Plein écran",
        &mut settings.fullscreen,
    );
    changed |= ui.toggle(
        id("pixel_art"),
        rows[2],
        "Effet pixel",
        &mut settings.pixel_art,
    );
    changed |= ui.toggle(
        id("filtering"),
        rows[3],
        "Textures lissées",
        &mut settings.filtering,
    );

    let step = ui.selector(
        id("fps"),
        rows[4],
        "Images par seconde",
        &match settings.framerate_limit {
            0 => "VSync".to_string(),
            n if n < 0 => "Illimité".to_string(),
            n => format!("{n} FPS"),
        },
    );
    if step != 0 {
        settings.framerate_limit = step_value(
            &[0, 30, 60, 120, 144, 240, -1],
            settings.framerate_limit,
            step,
        );
        changed = true;
    }

    changed
}

fn audio(ui: &mut Ui, settings: &mut Settings, content: Rect) -> bool {
    let rows = Rect {
        h: 2.0 * 84.0 + 18.0,
        ..content
    }
    .rows(2, 18.0);

    let music = ui.slider(id("music"), rows[0], "Musique", &mut settings.volume_music);
    let sounds = ui.slider(
        id("sounds"),
        rows[1],
        "Effets sonores",
        &mut settings.volume_sfx,
    );
    if sounds {
        ui.sound(SoundEffect::BonusPickup);
    }

    music || sounds
}
