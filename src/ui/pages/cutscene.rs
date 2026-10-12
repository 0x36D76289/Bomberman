use crate::{
    app_state::{AppState, Ctx, Transition, replace_or_menu},
    audio::BackgroundMusic,
    game::{game_state::GameState, theme::Theme},
    graphics::text::{Font, FontId},
    settings::save::SaveState,
    ui::{
        UiState,
        draw::{Align, TextStyle},
        layout::Rect,
        style,
        textures::{Tint, UiTex},
        widgets::Action,
    },
};
use glam::{Vec2, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scene {
    Intro,
    World(Theme),
    Ending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutsceneNext {
    Level(SaveState),
    Credits(u32),
}

type Slide = (&'static str, &'static [&'static str]);

impl Scene {
    pub fn world_intro(level: u32) -> Option<Self> {
        match level {
            2 => Some(Scene::World(Theme::Desert)),
            3 => Some(Scene::World(Theme::Snow)),
            4 => Some(Scene::World(Theme::Castle)),
            5 => Some(Scene::World(Theme::Volcano)),
            _ => None,
        }
    }

    pub fn music(self) -> BackgroundMusic {
        match self {
            Scene::World(theme) => theme.music(),
            _ => BackgroundMusic::Ending,
        }
    }

    fn theme(self) -> Theme {
        match self {
            Scene::Intro => Theme::Grass,
            Scene::World(theme) => theme,
            Scene::Ending => Theme::Volcano,
        }
    }

    fn slides(self) -> &'static [Slide] {
        match self {
            Scene::Intro => &[
                (
                    "LE ROYAUME DES BOMBES",
                    &[
                        "Le paisible Royaume des Bombes vivait heureux,",
                        "ses feux d'artifice illuminant chaque nuit.",
                    ],
                ),
                (
                    "UNE NUIT SOMBRE",
                    &[
                        "Une nuit, le cruel Seigneur Oozi vola la Mèche d'Or,",
                        "le cœur de toutes les bombes du royaume.",
                    ],
                ),
                (
                    "L'INVASION",
                    &[
                        "Ses créatures envahissent désormais le pays,",
                        "des vertes prairies jusqu'au Cœur du Volcan.",
                    ],
                ),
                (
                    "UN HÉROS",
                    &[
                        "Un seul héros porte encore une bombe...",
                        "Bomberman, le royaume compte sur toi !",
                    ],
                ),
            ],
            Scene::World(Theme::Desert) => &[(
                "DÉSERT DES DUNES",
                &[
                    "Les prairies sont de nouveau libres.",
                    "Les traces du Seigneur Oozi mènent aux sables brûlants...",
                ],
            )],
            Scene::World(Theme::Snow) => &[(
                "PICS GELÉS",
                &[
                    "Au-delà du désert se dressent les montagnes gelées.",
                    "Attention, les créatures y sont plus rapides.",
                ],
            )],
            Scene::World(Theme::Castle) => &[(
                "VIEUX CHÂTEAU",
                &[
                    "Au-delà des pics gelés se dresse le château du Seigneur Oozi.",
                    "Ses couloirs grouillent de ses meilleurs gardes.",
                ],
            )],
            Scene::World(_) => &[(
                "CŒUR DU VOLCAN",
                &[
                    "Le Seigneur Oozi se cache au cœur du volcan",
                    "avec la Mèche d'Or. Il faudra plusieurs bombes pour le vaincre !",
                ],
            )],
            Scene::Ending => &[
                (
                    "VICTOIRE !",
                    &[
                        "Le Seigneur Oozi est vaincu",
                        "et la Mèche d'Or a retrouvé sa place.",
                    ],
                ),
                (
                    "ENFIN LA PAIX",
                    &[
                        "Les feux d'artifice illuminent à nouveau le Royaume des Bombes.",
                        "Merci, Bomberman !",
                    ],
                ),
            ],
        }
    }
}

const TYPING_SPEED: f32 = 45.0;

pub fn backdrop(scene: Scene) -> GameState {
    GameState::backdrop(crate::resources(), scene.theme())
}

const TEXT_SIZES: [f32; 4] = [40.0, 36.0, 32.0, 28.0];

fn layout_lines(font: &Font, lines: &[&str], width: f32) -> (f32, Vec<String>) {
    let paragraph = lines.join(" ");
    let mut layout = Vec::new();
    for size in TEXT_SIZES {
        if lines
            .iter()
            .all(|line| font.line_width(line, size) <= width)
        {
            return (size, lines.iter().map(|line| line.to_string()).collect());
        }
        layout = font.wrap(&paragraph, size, width);
        if layout.len() <= 3 {
            return (size, layout);
        }
    }
    (TEXT_SIZES[TEXT_SIZES.len() - 1], layout)
}

fn text_length(lines: &[String]) -> usize {
    lines.iter().map(|line| line.chars().count() + 1).sum()
}

pub fn tick(
    state: &mut UiState,
    ctx: &mut Ctx,
    scene: Scene,
    slide: &mut usize,
    timer: &mut f32,
    next: CutsceneNext,
) -> Transition {
    let mut ui = super::begin(state, ctx);

    let screen = ui.screen();
    let slides = scene.slides();

    *slide = (*slide).min(slides.len() - 1);
    *timer += ctx.delta;

    let current = slides[*slide];
    let boxed = Rect::new((screen.x - 1300.0) / 2.0, screen.y - 430.0, 1300.0, 280.0);
    let (text_size, lines) =
        layout_lines(ui.dl.font(FontId::Title), current.1, 1300.0 - 120.0 - 80.0);
    let revealed = (*timer * TYPING_SPEED) as usize;
    let length = text_length(&lines);
    let mut finished = ui.back_pressed();
    let advance = ui.input.confirm || ui.input.mouse_released;

    if advance && revealed < length {
        *timer = length as f32 / TYPING_SPEED;
    } else if advance || *timer > length as f32 / TYPING_SPEED + 3.5 {
        if *slide + 1 < slides.len() {
            *slide += 1;
            *timer = 0.0;
            ui.sound(crate::audio::SoundEffect::MenuMove);
        } else {
            finished = true;
        }
    }

    let bar_height = 110.0;

    ui.dl.rect(
        Rect::new(0.0, 0.0, screen.x, bar_height),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    );
    ui.dl.rect(
        Rect::new(0.0, screen.y - bar_height, screen.x, bar_height),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    );

    ui.dl.frame(UiTex::Panel, boxed, Vec4::ONE, 0.7);
    let tag = Rect::new(
        boxed.x + 40.0,
        boxed.y - 36.0,
        ui.dl.text_width(
            current.0,
            &TextStyle::new(FontId::Title, 38.0, style::WHITE),
        ) + 60.0,
        72.0,
    );
    ui.dl
        .frame(UiTex::Button(Tint::Yellow), tag, Vec4::ONE, 0.45);
    ui.dl.text(
        current.0,
        tag.center() - Vec2::new(0.0, 3.0),
        TextStyle::new(FontId::Title, 38.0, style::WHITE)
            .shadow(style::fade(style::INK, 0.4), Vec2::new(0.0, 3.0)),
    );

    let mut remaining = revealed;
    for (i, line) in lines.iter().enumerate() {
        ui.dl.text(
            &line.chars().take(remaining).collect::<String>(),
            Vec2::new(
                boxed.x + 60.0,
                boxed.y + 95.0 + i as f32 * (text_size * 1.5),
            ),
            TextStyle::new(FontId::Title, text_size, style::TEXT_DARK).align(Align::Left),
        );
        remaining = remaining.saturating_sub(line.chars().count() + 1);
    }
    if revealed >= length {
        ui.dl.image_fit(
            UiTex::ArrowRight,
            Rect::centered(
                Vec2::new(boxed.right() - 60.0, boxed.bottom() - 50.0),
                Vec2::splat(44.0),
            ),
            Vec4::ONE,
        );
    }

    for i in 0..slides.len() {
        ui.dl.image_fit(
            UiTex::Star,
            Rect::centered(
                Vec2::new(
                    screen.x / 2.0 + (i as f32 - (slides.len() - 1) as f32 / 2.0) * 34.0,
                    screen.y - 125.0,
                ),
                Vec2::splat(if i == *slide { 30.0 } else { 20.0 }),
            ),
            if i == *slide {
                style::YELLOW
            } else {
                style::fade(style::WHITE, 0.4)
            },
        );
    }
    ui.prompts(&[(Action::Confirm, "Suivant"), (Action::Back, "Passer")]);
    ui.end(ctx.audio);

    if !finished {
        return (None, 0);
    }
    match next {
        CutsceneNext::Level(run) => {
            replace_or_menu(GameState::new_campaign(run).map(AppState::game), 1)
        }
        CutsceneNext::Credits(_) => {
            ctx.settings.save();
            (Some(AppState::credits()), 1)
        }
    }
}
