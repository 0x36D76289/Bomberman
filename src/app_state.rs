use crate::{
    audio::AudioManager,
    game::{game_settings::GameSettings, game_state::GameState, resources::Resources},
    input::{event::InputEvent, input::Input},
    settings::settings::Settings,
    ui::{UiState, pages::game_settings::UIGameSettings, pages::stage_clear::STAGE_CLEAR_DURATION},
};

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
    Pause,
    LevelSelect,
    GameSettings(UIGameSettings),
    Settings {
        selected_player: usize,
    },
    Binds {
        player: usize,
        waiting: isize,
    },
    GameOver,
    StageClear {
        timer: f32,
        next_level: u32,
        lives: u32,
    },
    MultiplayerEndScreen,
}

impl AppState {
    pub fn binds(player: usize, ratio: f32) -> Self {
        Self {
            state: AppStateEnum::Binds {
                player,
                waiting: -1,
            },
            game: None,
            ui: Some(UiState::binds(player, ratio)),
        }
    }

    pub fn game(game_state: GameState) -> Self {
        Self {
            state: AppStateEnum::Game,
            game: Some(game_state),
            ..Default::default()
        }
    }
    pub fn game_over() -> Self {
        Self {
            state: AppStateEnum::GameOver,
            game: None,
            ui: Some(UiState::game_over()),
        }
    }

    pub fn game_settings(resources: &Resources, player_count: u8) -> Self {
        Self {
            state: AppStateEnum::GameSettings(UIGameSettings::corners(player_count)),
            game: Some(GameState::new_settings_preview(
                GameSettings::default(),
                resources,
            )),
            ui: Some(UiState::game_settings(player_count)),
        }
    }
    pub fn level_select() -> Self {
        Self {
            state: AppStateEnum::LevelSelect,
            game: None,
            ui: Some(UiState::level_select()),
        }
    }
    pub fn main_menu() -> Self {
        Self {
            state: AppStateEnum::MainMenu,
            game: None,
            ui: Some(UiState::main_menu()),
        }
    }
    pub fn multiplayer_end_screen(winners: Vec<u32>) -> Self {
        Self {
            state: AppStateEnum::MultiplayerEndScreen,
            game: None,
            ui: Some(UiState::multiplayer_end_screen(winners)),
        }
    }

    pub fn pause() -> Self {
        Self {
            state: AppStateEnum::Pause,
            game: None,
            ui: Some(UiState::pause()),
        }
    }
    pub fn settings() -> Self {
        Self {
            state: AppStateEnum::Settings { selected_player: 0 },
            game: None,
            ui: Some(UiState::settings()),
        }
    }
    pub fn stage_clear(settings: &mut Settings, level: u32, lives: u32) -> Self {
        Self {
            state: AppStateEnum::StageClear {
                timer: STAGE_CLEAR_DURATION,
                next_level: level + 1,
                lives,
            },
            game: None,
            ui: Some(UiState::stage_clear(settings, level, lives)),
        }
    }

    pub fn tick(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        events: &Vec<InputEvent>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
        settings: &mut Settings,
        ratio: f32,
    ) -> (Option<AppState>, u8) {
        match &mut self.state {
            AppStateEnum::Game => {
                self.game
                    .as_mut()
                    .unwrap()
                    .tick(delta, inputs, resources, audio_manager, settings)
            }
            AppStateEnum::MainMenu => {
                self.ui
                    .as_mut()
                    .unwrap()
                    .main_menu_tick(inputs, audio_manager, resources, settings)
            }
            AppStateEnum::LevelSelect => self
                .ui
                .as_mut()
                .unwrap()
                .level_select_tick(inputs, audio_manager),
            AppStateEnum::GameSettings(ui_game_settings) => {
                let (game, ui) = match (self.game.as_mut(), self.ui.as_mut()) {
                    (Some(game), Some(ui)) => (game, ui),
                    _ => panic!(
                        "Tried to tick game settings but app state doesnt contain a gamestate or a uistate"
                    ),
                };
                let old_settings = *ui_game_settings;
                match ui.game_settings_tick(delta, inputs, ui_game_settings) {
                    None => {
                        if old_settings != *ui_game_settings {
                            let err = game.update_from_ui_settings(ui_game_settings, resources);
                            if err.is_err() {
                                ui.set_error(
                                    "Map creation failed, try ajusting settings".to_string(),
                                    ui_game_settings,
                                );
                            }
                        }
                        (None, 0)
                    }
                    Some(enter_pressed) => {
                        if enter_pressed {
                            (
                                Some(AppState::game(
                                    GameState::new_multiplayer_from_map(
                                        resources,
                                        GameSettings::default(),
                                        game.get_map().clone(),
                                    )
                                    .unwrap(),
                                )),
                                0,
                            )
                        } else {
                            (None, 1)
                        }
                    }
                }
            }
            AppStateEnum::Settings { selected_player } => {
                self.ui
                    .as_mut()
                    .unwrap()
                    .settings_tick(inputs, settings, selected_player, ratio)
            }
            AppStateEnum::Binds { player, waiting } => self
                .ui
                .as_mut()
                .unwrap()
                .binds_tick(inputs, events, settings, player, waiting),
            AppStateEnum::Pause => {
                self.ui
                    .as_mut()
                    .unwrap()
                    .pause_tick(inputs, resources, audio_manager)
            }
            AppStateEnum::GameOver => self
                .ui
                .as_mut()
                .unwrap()
                .game_over_tick(inputs, audio_manager),
            AppStateEnum::StageClear {
                timer,
                next_level,
                lives,
            } => self
                .ui
                .as_mut()
                .unwrap()
                .stage_clear_tick(delta, timer, next_level, lives),
            AppStateEnum::MultiplayerEndScreen => self
                .ui
                .as_ref()
                .unwrap()
                .multiplayer_end_screen_tick(inputs),
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self.state {
            AppStateEnum::Game => false,
            AppStateEnum::Pause => true,
            AppStateEnum::GameOver => true,
            AppStateEnum::MainMenu => false,
            AppStateEnum::LevelSelect => false,
            AppStateEnum::GameSettings(_) => false,
            AppStateEnum::Binds { .. } => false,
            AppStateEnum::Settings { .. } => false,
            AppStateEnum::StageClear { .. } => true,
            AppStateEnum::MultiplayerEndScreen => false,
        }
    }
}
