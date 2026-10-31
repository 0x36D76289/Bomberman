use crate::{
    audio::{AudioManager, BackgroundMusic},
    game::{
        game_state::{ClearStats, GameState, RestartInfo},
        lobby::LobbySettings,
        resources::Resources,
    },
    input::{event::InputEvent, input::Input},
    settings::{
        save::{GameDifficulty, SaveState},
        settings::Settings,
    },
    ui::{
        UiState,
        input::UiInput,
        pages::{
            self,
            cutscene::{CutsceneNext, Scene},
            settings::SettingsPage,
        },
    },
};
use glam::Vec2;

pub const POP_TO_MENU: u8 = u8::MAX;

pub type Transition = (Option<AppState>, u8);

pub fn replace_or_menu(state: Option<AppState>, pop: u8) -> Transition {
    match state {
        Some(state) => (Some(state), pop),
        None => (None, POP_TO_MENU),
    }
}

pub struct Ctx<'a> {
    pub delta: f32,
    pub inputs: &'a Vec<Input>,
    pub events: &'a Vec<InputEvent>,
    pub ui: &'a UiInput,
    pub resources: &'static Resources,
    pub audio: &'a mut AudioManager,
    pub settings: &'a mut Settings,
    pub screen: Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub state: AppStateEnum,
    pub game: Option<GameState>,
    pub ui: Option<UiState>,
}

#[derive(Debug, Clone, Default)]
pub enum AppStateEnum {
    Game,
    #[default]
    MainMenu,
    NewGame {
        difficulty: GameDifficulty,
        random_layout: bool,
    },
    StageMap {
        run: SaveState,
    },
    Lobby(LobbySettings),
    Settings(SettingsPage),
    Pause {
        restart: RestartInfo,
        confirm: Option<pages::pause::PauseConfirm>,
    },
    GameOver {
        run: SaveState,
    },
    StageClear {
        run: SaveState,
        stats: ClearStats,
        name: String,
    },
    MultiplayerEnd {
        winners: Vec<u32>,
        restart: RestartInfo,
    },
    Cutscene {
        scene: Scene,
        slide: usize,
        timer: f32,
        next: CutsceneNext,
    },
    Credits,
}

impl AppState {
    fn page(state: AppStateEnum) -> Self {
        Self {
            state,
            game: None,
            ui: Some(UiState::default()),
        }
    }

    pub fn game(game_state: GameState) -> Self {
        Self {
            state: AppStateEnum::Game,
            game: Some(game_state),
            ui: Some(UiState::default()),
        }
    }

    pub fn level_start(settings: &mut Settings, run: SaveState, with_story: bool) -> Option<Self> {
        let game = GameState::new_campaign(run)?;
        settings.save_progress(run);
        if with_story {
            if run.level == 1 && !settings.intro_seen {
                settings.intro_seen = true;
                settings.save();
                return Some(Self::cutscene(Scene::Intro, CutsceneNext::Level(run)));
            }
            if let Some(scene) = Scene::world_intro(run.level) {
                return Some(Self::cutscene(scene, CutsceneNext::Level(run)));
            }
        }
        Some(Self::game(game))
    }

    pub fn main_menu() -> Self {
        Self {
            state: AppStateEnum::MainMenu,
            game: Some(GameState::backdrop(
                crate::resources(),
                crate::game::theme::Theme::Grass,
            )),
            ui: Some(UiState::default()),
        }
    }

    pub fn new_game() -> Self {
        Self::page(AppStateEnum::NewGame {
            difficulty: GameDifficulty::Normal,
            random_layout: false,
        })
    }

    pub fn stage_map(run: SaveState) -> Self {
        Self::page(AppStateEnum::StageMap { run })
    }

    pub fn lobby(resources: &Resources, humans: u8) -> Self {
        let lobby = LobbySettings::new(crate::game::lobby::LobbyPreset::Corners, humans);
        let mut ui = UiState::default();
        ui.render_info.drawn_first = true;
        Self {
            state: AppStateEnum::Lobby(lobby),
            game: Some(GameState::lobby_preview(&lobby, resources)),
            ui: Some(ui),
        }
    }

    pub fn settings(in_game: bool) -> Self {
        Self::page(AppStateEnum::Settings(SettingsPage::new(in_game)))
    }

    pub fn pause(restart: RestartInfo) -> Self {
        Self::page(AppStateEnum::Pause {
            restart,
            confirm: None,
        })
    }

    pub fn game_over(run: SaveState) -> Self {
        Self::page(AppStateEnum::GameOver { run })
    }

    pub fn stage_clear(
        settings: &mut Settings,
        run: SaveState,
        name: String,
        stats: ClearStats,
    ) -> Self {
        pages::stage_clear::save(settings, run);
        Self::page(AppStateEnum::StageClear { run, stats, name })
    }

    pub fn multiplayer_end_screen(winners: Vec<u32>, restart: RestartInfo) -> Self {
        Self::page(AppStateEnum::MultiplayerEnd { winners, restart })
    }

    pub fn cutscene(scene: Scene, next: CutsceneNext) -> Self {
        Self {
            state: AppStateEnum::Cutscene {
                scene,
                slide: 0,
                timer: 0.0,
                next,
            },
            game: Some(pages::cutscene::backdrop(scene)),
            ui: Some(UiState::default()),
        }
    }

    pub fn credits() -> Self {
        Self::page(AppStateEnum::Credits)
    }

    pub fn tick(&mut self, ctx: &mut Ctx) -> Transition {
        let ui = self.ui.get_or_insert_with(UiState::default);
        match &mut self.state {
            AppStateEnum::Game => {
                let Some(game) = self.game.as_mut() else {
                    return (None, POP_TO_MENU);
                };
                let ret = game.tick(
                    ctx.delta,
                    ctx.inputs,
                    ctx.resources,
                    ctx.audio,
                    ctx.settings,
                );
                pages::hud::draw(ui, game, ctx);
                ret
            }
            AppStateEnum::MainMenu => pages::main_menu::tick(ui, ctx),
            AppStateEnum::NewGame {
                difficulty,
                random_layout,
            } => pages::new_game::tick(ui, ctx, difficulty, random_layout),
            AppStateEnum::StageMap { run } => pages::stage_map::tick(ui, ctx, *run),
            AppStateEnum::Lobby(lobby) => {
                let old = *lobby;
                let ret = pages::lobby::tick(ui, ctx, lobby, self.game.as_mut());
                if *lobby != old
                    && let Some(preview) = self.game.as_mut()
                {
                    preview.update_from_lobby(lobby, ctx.resources);
                }
                ret
            }
            AppStateEnum::Settings(page) => page.tick(ui, ctx),
            AppStateEnum::Pause { restart, confirm } => {
                pages::pause::tick(ui, ctx, restart, confirm)
            }
            AppStateEnum::GameOver { run } => pages::game_over::tick(ui, ctx, *run),
            AppStateEnum::StageClear { run, stats, name } => {
                pages::stage_clear::tick(ui, ctx, *run, *stats, name)
            }
            AppStateEnum::MultiplayerEnd { winners, restart } => {
                pages::mp_end::tick(ui, ctx, winners, restart)
            }
            AppStateEnum::Cutscene {
                scene,
                slide,
                timer,
                next,
            } => pages::cutscene::tick(ui, ctx, *scene, slide, timer, *next),
            AppStateEnum::Credits => pages::credits::tick(ui, ctx),
        }
    }

    pub fn is_transparent(&self) -> bool {
        match &self.state {
            AppStateEnum::Pause { .. }
            | AppStateEnum::GameOver { .. }
            | AppStateEnum::StageClear { .. }
            | AppStateEnum::MultiplayerEnd { .. } => true,
            AppStateEnum::Settings(page) => page.in_game,
            _ => false,
        }
    }

    pub fn is_gameplay(&self) -> bool {
        matches!(self.state, AppStateEnum::Game)
    }

    pub fn music(&self) -> Option<Option<BackgroundMusic>> {
        match &self.state {
            AppStateEnum::MainMenu => Some(Some(BackgroundMusic::Menu)),
            AppStateEnum::Settings(page) if !page.in_game => Some(Some(BackgroundMusic::Menu)),
            AppStateEnum::NewGame { .. }
            | AppStateEnum::StageMap { .. }
            | AppStateEnum::Lobby(_)
            | AppStateEnum::MultiplayerEnd { .. } => Some(Some(BackgroundMusic::Select)),
            AppStateEnum::Game => self.game.as_ref().map(|game| Some(game.theme().music())),
            AppStateEnum::GameOver { .. } | AppStateEnum::StageClear { .. } => Some(None),
            AppStateEnum::Pause { .. } | AppStateEnum::Settings(_) => None,
            AppStateEnum::Cutscene { scene, .. } => Some(Some(scene.music())),
            AppStateEnum::Credits => Some(Some(BackgroundMusic::Ending)),
        }
    }

    pub fn update_music(&self, audio_manager: &mut AudioManager) {
        match self.music() {
            Some(Some(music)) => audio_manager.play_background_music(music),
            Some(None) => audio_manager.stop_background_music(),
            None => (),
        }
    }
}
