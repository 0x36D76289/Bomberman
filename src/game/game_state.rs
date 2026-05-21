use crate::app_state::AppState;
use crate::game::bomb::{Bomb, BombEvents, BombState};
use crate::game::camera::Camera;
use crate::game::collision::Collision;
use crate::game::enemy::{Enemy, EnemyBehavior};
use crate::game::game_settings::GameSettings;
use crate::game::map::map::{LevelData, Map};
use crate::game::map::map_element::MapElement;
use crate::game::map::map_settings::MapSettings;
use crate::game::player::Player;
use crate::game::powerup::PowerUp;
use crate::game::resources::{ResourceName, Resources};
use crate::graphics::object::Object;
use crate::graphics::transform::Transform;
use crate::graphics::{GlobalUbo, LightInfo, StateRenderInfo};
use crate::input::input::Input;
use crate::input::input_state::InputState;
use crate::input::input_vec::{GetOrDefault, MenuInput};
use crate::settings::{save::GameDifficulty, settings::Settings};
use crate::ui::pages::game_settings::UIGameSettings;
use crate::{audio::AudioManager, audio::SoundEffect};
use glam::{Vec2, Vec3, Vec4};
use rand::random_range;
use std::error::Error;
use std::vec::Vec;

/// This enum is used to select what tick function to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Multiplayer,
    Campaign,
}

#[derive(Debug, Clone, Copy)]
struct ScoreRules {
    enemy_kill: u32,
    breakable_destroyed: u32,
    powerup_pickup: u32,
    level_clear: u32,
    time_bonus_seconds: u32,
    time_bonus_per_second: u32,
    time_bonus_cap: u32,
}

impl ScoreRules {
    fn time_bonus(&self, elapsed: f32) -> u32 {
        if self.time_bonus_per_second == 0 || self.time_bonus_seconds == 0 {
            return 0;
        }
        let elapsed_seconds = elapsed.max(0.0) as u32;
        if elapsed_seconds >= self.time_bonus_seconds {
            return 0;
        }
        let remaining = self.time_bonus_seconds - elapsed_seconds;
        (remaining * self.time_bonus_per_second).min(self.time_bonus_cap)
    }
}

const SCORE_RULES: ScoreRules = ScoreRules {
    enemy_kill: 100,
    breakable_destroyed: 10,
    powerup_pickup: 25,
    level_clear: 300,
    time_bonus_seconds: 120,
    time_bonus_per_second: 2,
    time_bonus_cap: 240,
};

#[derive(Default, Debug, Clone, Copy)]
struct TickEvents {
    breakables_destroyed: u32,
    enemies_killed: u32,
    powerups_picked: u32,
}

impl TickEvents {
    fn add_bomb_events(&mut self, events: BombEvents) {
        self.breakables_destroyed = self
            .breakables_destroyed
            .saturating_add(events.breakables_destroyed);
        self.enemies_killed = self.enemies_killed.saturating_add(events.enemies_killed);
    }
}

#[derive(Debug, Clone)]
struct MpTickResult {
    winners: Option<Vec<u32>>,
    events: TickEvents,
}

#[derive(Debug, Clone, Copy)]
struct DifficultyPreset {
    enemy_speed_multiplier: f32,
    decision_interval: f32,
    aggressive_pct: u32,
    coward_pct: u32,
    extra_enemies: u32,
}

impl DifficultyPreset {
    fn for_level(level: u32, difficulty: GameDifficulty) -> Self {
        let level_index = level.saturating_sub(1);
        let mut enemy_speed_multiplier = (1.0 + level_index as f32 * 0.08).min(1.6);
        let mut decision_interval = (0.65 - level_index as f32 * 0.04).max(0.3);
        let mut aggressive_pct = (30 + level_index * 5).min(60);
        let coward_pct = 20;
        let mut extra_enemies = level_index.min(4);

        match difficulty {
            GameDifficulty::Easy => {
                enemy_speed_multiplier *= 0.85;
                decision_interval *= 1.2;
                aggressive_pct = aggressive_pct.saturating_sub(10);
                extra_enemies = extra_enemies.saturating_sub(1);
            }
            GameDifficulty::Normal => {}
            GameDifficulty::Hard => {
                enemy_speed_multiplier *= 1.15;
                decision_interval *= 0.85;
                aggressive_pct = (aggressive_pct + 10).min(80);
                extra_enemies = (extra_enemies + 1).min(6);
            }
        }

        enemy_speed_multiplier = enemy_speed_multiplier.clamp(0.6, 2.0);
        decision_interval = decision_interval.clamp(0.25, 0.9);

        Self {
            enemy_speed_multiplier,
            decision_interval,
            aggressive_pct,
            coward_pct,
            extra_enemies,
        }
    }

    fn pick_behavior(&self) -> EnemyBehavior {
        let roll = random_range(0..=99);
        let aggressive = self.aggressive_pct.min(100);
        let coward = self.coward_pct.min(100 - aggressive);
        if roll < aggressive {
            EnemyBehavior::Aggressive
        } else if roll < aggressive + coward {
            EnemyBehavior::Coward
        } else {
            EnemyBehavior::Wander
        }
    }
}

/// The [CampaignProgress] mirrors the [SaveState](crate::settings::save::SaveState) but is a temporary format
#[derive(Debug, Clone, Default)]
pub struct CampaignProgress {
    pub level: u32,
    pub lives: u32,
    /// The score of the ongoing campaign
    pub score: u32,
    pub level_time: f32,
    pub difficulty: GameDifficulty,
}

/// The return type of [GameState]'s singleplayer tick function
#[derive(Debug, Clone)]
pub enum GameTickResult {
    /// The level is still ongoing
    None,
    /// The [Player] failed
    GameOver,
    /// The [Player] killed all the [Enemy]s and got to the [exit point](MapElement::Exit)
    LevelComplete,
}

/// The entire game logic's data, receives input and updates game state
#[derive(Debug, Clone)]
pub struct GameState {
    /// Whether this is a singleplayer or multiplayer game
    mode: GameMode,
    /// The live state of the current campaign, None in multiplayer
    campaign_progress: Option<CampaignProgress>,
    /// The list of players at game start
    players: Vec<Player>,
    /// The list of enemies for singleplayer levels
    enemies: Vec<Enemy>,
    // TODO: does this double up on the exit mapelement ? add doc
    exit_pos: Vec2,
    /// becomes true once all enemies are killed, lets the player complete a level
    exit_revealed: bool,
    /// The inputs of every player after processing events
    game_inputs: Vec<Input>,
    // TODO: if unused by AI input then remove else doc
    nb_humans: u32,
    /// The list of bombs/explosions currently in game
    bombs: Vec<Bomb>,
    /// The list of PowerUps that [Player]s can pick up
    power_ups: Vec<PowerUp>,
    /// The map the game takes place in
    map: Map,
    /// The camera data changes how the game is rendered
    camera: Transform,
    /// The lighting of the game
    light: LightInfo,
    /// The [ids](Player::id) of currently alive [Player]s
    alive_players: Vec<u32>,
    // TODO: doc
    pub render_info: StateRenderInfo,
}

impl GameState {
    /// Initiates a new game for multiplayer from given settings
    pub fn new_multiplayer(
        resources: &Resources,
        settings: GameSettings,
    ) -> Result<Self, Box<dyn Error>> {
        let Some(map) = MapSettings::new_map(settings.map_settings, resources) else {
            return Err("Map creation fail".into());
        };
        let nb_humans = settings.nb_humans;
        let players = Self::create_players(&map, resources, nb_humans, settings.nb_bots);
        let alive_players = players.iter().map(|player| player.id).collect();
        let game_inputs = vec![Input::default(); players.len()];

        let camera = Transform {
            translation: Vec3::new(map.width as f32 / 2.0, -1.0, map.height as f32 / 2.0),
            scale: Vec3::ONE,
            rotation: Vec3::new(-1.25, 0.0, 0.0),
        };

        let light = LightInfo {
            ambient_light_color: Vec4::ONE.with_w(0.8),
            direction_to_light: Vec3::new(0.0, -3.0, 1.0).normalize(),
            directional_light_color: Vec4::ONE.with_w(0.6),
        };

        let render_info = StateRenderInfo {
            drawn_first: true,
            ..Default::default()
        };

        Ok(Self {
            mode: GameMode::Multiplayer,
            campaign_progress: None,
            players,
            enemies: Vec::new(),
            exit_pos: Vec2::ZERO,
            exit_revealed: false,
            game_inputs,
            nb_humans,
            bombs: Vec::<Bomb>::new(),
            power_ups: Vec::<PowerUp>::new(),
            map,
            camera,
            light,
            render_info,
            alive_players,
        })
    }

    /// begins a new campaign, used by the New Game button
    pub fn new_campaign(
        level: u32,
        lives: u32,
        score: u32,
        difficulty: GameDifficulty,
    ) -> Option<Self> {
        let resources_to_load_map = unsafe {
            (*std::ptr::addr_of!(crate::GLOBAL_RESOURCES))
                .as_ref()
                .unwrap()
        };

        let LevelData {
            map,
            player_spawn,
            enemy_spawns,
            exit_pos,
        } = Map::from_file(level, resources_to_load_map)?;

        let mut players = Vec::new();
        players.push(Player::new(
            0,
            player_spawn,
            super::direction::Direction::Down,
            resources_to_load_map,
            true,
        ));
        let alive_players = players.iter().map(|player| player.id).collect();

        let difficulty_preset = DifficultyPreset::for_level(level, difficulty);
        let mut enemy_positions = enemy_spawns;
        let mut avoid_positions = vec![player_spawn];
        avoid_positions.extend(enemy_positions.iter().copied());

        for _ in 0..difficulty_preset.extra_enemies {
            if let Some(pos) = map.find_random_empty(&avoid_positions, 2.0, 40) {
                enemy_positions.push(pos);
                avoid_positions.push(pos);
            }
        }

        let mut enemies = Vec::new();
        for (i, spawn) in enemy_positions.iter().enumerate() {
            let behavior = difficulty_preset.pick_behavior();
            enemies.push(Enemy::new(
                i as u32,
                *spawn,
                behavior,
                difficulty_preset.enemy_speed_multiplier,
                difficulty_preset.decision_interval,
                resources_to_load_map,
            ));
        }

        let camera = Transform {
            translation: Vec3::new(map.width as f32 / 2.0, -1.0, map.height as f32 / 2.0),
            scale: Vec3::ONE,
            rotation: Vec3::new(-1.25, 0.0, 0.0),
        };

        let light = LightInfo {
            ambient_light_color: Vec4::ONE.with_w(0.8),
            direction_to_light: Vec3::new(0.0, -3.0, 1.0).normalize(),
            directional_light_color: Vec4::ONE.with_w(0.6),
        };

        let render_info = StateRenderInfo {
            drawn_first: true,
            ..Default::default()
        };

        Some(Self {
            mode: GameMode::Campaign,
            campaign_progress: Some(CampaignProgress {
                level,
                lives,
                score,
                level_time: 0.0,
                difficulty,
            }),
            players,
            enemies,
            exit_pos,
            exit_revealed: false,
            game_inputs: vec![Input::default(); 1],
            nb_humans: 1,
            bombs: Vec::new(),
            power_ups: Vec::new(),
            map,
            camera,
            light,
            render_info,
            alive_players,
        })
    }

    /// Creates a partial gamestate, used in multiplayer settings selection for map preview
    pub fn new_settings_preview(settings: GameSettings, resources: &Resources) -> Self {
        Self {
            render_info: StateRenderInfo {
                top_left_coord: [-1.0, -0.3],
                bottom_right_coord: [0.0, 0.7],
                drawn_first: false,
            },
            players: Vec::new(),
            enemies: Vec::new(),
            ..GameState::new_multiplayer(resources, settings).unwrap()
        }
    }

    /// Creates a multiplayer game from an existing map
    /// Used when starting a multiplayer game
    pub fn new_multiplayer_from_map(
        resources: &Resources,
        settings: GameSettings,
        map: Map,
    ) -> Self {
        let nb_humans = settings.nb_humans;
        let players = Self::create_players(&map, resources, nb_humans, settings.nb_bots);
        let alive_players = players.iter().map(|player| player.id).collect();
        let game_inputs = vec![Input::default(); players.len()];

        let camera = Transform {
            translation: Vec3::new(map.width as f32 / 2.0, -1.0, map.height as f32 / 2.0),
            scale: Vec3::ONE,
            rotation: Vec3::new(-1.25, 0.0, 0.0),
        };

        let light = LightInfo {
            ambient_light_color: Vec4::ONE.with_w(0.8),
            direction_to_light: Vec3::new(0.0, -3.0, 1.0).normalize(),
            directional_light_color: Vec4::ONE.with_w(0.6),
        };

        let render_info = StateRenderInfo {
            drawn_first: true,
            ..Default::default()
        };

        Self {
            mode: GameMode::Multiplayer,
            campaign_progress: None,
            players,
            enemies: Vec::new(),
            exit_pos: Vec2::ZERO,
            exit_revealed: false,
            game_inputs,
            nb_humans,
            bombs: Vec::<Bomb>::new(),
            power_ups: Vec::<PowerUp>::new(),
            map,
            camera,
            light,
            render_info,
            alive_players,
        }
    }

    /// Creates all the [Player] objects for a GameState and prepares them to be manipulated by
    /// AI or humans
    fn create_players(
        map: &Map,
        resources: &Resources,
        nb_humans: u32,
        nb_bots: u32,
    ) -> Vec<Player> {
        let mut players = Vec::<Player>::new();
        let mut id: u32 = 0;
        for (i, spawn) in map.spawns.clone().iter().enumerate() {
            if i >= (nb_humans + nb_bots) as usize {
                break;
            }
            players.push(Player::new(
                id,
                Vec2 {
                    x: spawn.x as f32 + 0.5,
                    y: spawn.y as f32 + 0.5,
                },
                spawn.direction,
                resources,
                id < nb_humans,
            ));

            id += 1;
        }

        players
    }

    /// Creates a list of objects to render from the gamestate for the render function
    pub fn objects_to_render(&self) -> impl Iterator<Item = &Object> {
        let map_objects = self.map.iter().filter_map(|el| match el {
            MapElement::Empty => None,
            MapElement::Breakable(obj) => Some(obj),
            MapElement::Unbreakable(obj) => Some(obj),
            MapElement::Exit(obj) => Some(obj),
        });

        let floor_iter = std::iter::once(&self.map.floor);
        let players_objects = self.players.iter().filter_map(|p| p.object.as_ref());
        let enemy_objects = self.enemies.iter().filter_map(|e| e.object.as_ref());
        let bomb_objects = self.bombs.iter().flat_map(|b| &b.objects);
        let power_up_objects = self.power_ups.iter().map(|p| &p.object);

        map_objects
            .chain(floor_iter)
            .chain(players_objects)
            .chain(enemy_objects)
            .chain(bomb_objects)
            .chain(power_up_objects)
    }

    /// The main tick function of multiplayer games, simulates every event since the last frame
    fn mp_game_tick(
        &mut self,
        delta: f32,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> MpTickResult {
        let mut events = TickEvents::default();
        // tick bombs
        for i in 0..self.bombs.len() {
            let bombs_pos = self
                .bombs
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != i)
                .map(|(_, bomb)| bomb.position)
                .collect::<Vec<_>>();

            let bomb_events = self.bombs[i].tick(
                delta,
                &mut self.players,
                &mut self.enemies,
                &mut self.map,
                &mut self.power_ups,
                resources,
                audio_manager,
                &bombs_pos,
            );
            events.add_bomb_events(bomb_events);
        }
        for i in 0..self.bombs.len() {
            if self.bombs[i].state != BombState::Exploding {
                continue;
            }
            self.bombs[i].clone().chain_react(&mut self.bombs);
        }
        self.bombs.retain(|bomb| !bomb.despawn);
        for powerup in &mut self.power_ups {
            if powerup.tick(&mut self.players, audio_manager) {
                events.powerups_picked = events.powerups_picked.saturating_add(1);
            }
        }
        self.power_ups.retain(|powerup| !powerup.despawn);
        // for player in players: summon bomb if Pressed
        let player_poses = self
            .players
            .iter()
            .filter(|player| player.alive)
            .map(|player| (player.id, player.position))
            .collect::<Vec<_>>();
        for (i, player) in self.players.iter_mut().enumerate() {
            if !player.alive {
                continue;
            }
            if self.game_inputs.get_or_default(i).bomb() == InputState::Pressed
                && let Some(bomb) = player.create_bomb(&resources, &self.bombs, &player_poses)
            {
                audio_manager.play_sound_effect(crate::audio::SoundEffect::PutBomb);
                self.bombs.push(bomb)
            }
        }
        for (i, player) in self.players.iter_mut().enumerate() {
            player.player_move(
                self.game_inputs.get_or_default(i),
                delta,
                &self.map,
                &mut self.bombs,
            );
        }
        MpTickResult {
            winners: self.create_mp_ret(),
            events,
        }
    }

    /// Helper function to detect the end of multiplayer game
    fn create_mp_ret(&mut self) -> Option<Vec<u32>> {
        let alive_players: Vec<u32> = self
            .players
            .iter()
            .filter_map(|player| if player.alive { Some(player.id) } else { None })
            .collect();

        if alive_players.is_empty() {
            return Some(self.alive_players.clone());
        }
        if alive_players.len() == 1 {
            return Some(alive_players);
        }
        self.alive_players = alive_players;
        return None;
    }

    /// The main tick function of campaign level games, it simulates every event since the
    /// last frame
    fn campaign_tick(
        &mut self,
        delta: f32,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> GameTickResult {
        // Player is dead check for lives
        if !self.players[0].alive {
            if let Some(progress) = &mut self.campaign_progress {
                if progress.lives > 0 {
                    progress.lives -= 1;
                    // Respawn player
                    self.players[0].respawn(self.map.spawns[0].position(), resources);
                } else {
                    return GameTickResult::GameOver;
                }
            }
        }

        if let Some(progress) = &mut self.campaign_progress {
            progress.level_time += delta;
        }

        // Tick enemies
        let player_pos = self.players.get(0).filter(|p| p.alive).map(|p| p.position);
        for i in 0..self.enemies.len() {
            let (left, right) = self.enemies.split_at_mut(i);
            if let Some((current, right)) = right.split_first_mut() {
                let other_enemies: Vec<_> = left.iter().chain(right.iter()).cloned().collect();
                current.tick(delta, &self.map, player_pos, &self.bombs, &other_enemies);
            }

            if self.players[0].alive
                && self.enemies[i].alive
                && self.players[0]
                    .is_colliding_with(self.enemies[i].position, self.enemies[i].get_size())
            {
                self.players[0].kill();
                audio_manager.play_sound_effect(SoundEffect::PlayerDeath);
            }
        }

        // Tick bombs and other shared logic
        let mp_result = self.mp_game_tick(delta, resources, audio_manager);
        if let Some(progress) = &mut self.campaign_progress {
            let mut added_score: u32 = 0;
            added_score = added_score.saturating_add(
                mp_result
                    .events
                    .enemies_killed
                    .saturating_mul(SCORE_RULES.enemy_kill),
            );
            added_score = added_score.saturating_add(
                mp_result
                    .events
                    .breakables_destroyed
                    .saturating_mul(SCORE_RULES.breakable_destroyed),
            );
            added_score = added_score.saturating_add(
                mp_result
                    .events
                    .powerups_picked
                    .saturating_mul(SCORE_RULES.powerup_pickup),
            );
            progress.score = progress.score.saturating_add(added_score);
        }

        self.enemies.retain(|e| e.alive);
        if self.enemies.is_empty() && !self.exit_revealed {
            self.exit_revealed = true;
            println!(
                "All enemies defeated! Exit revealed at position: {:?}",
                self.exit_pos
            );
            let exit_obj = Object {
                model: resources.models[&ResourceName::Floor].clone(),
                texture: None,
                color: Vec3::new(0.2, 0.8, 0.2),
                transform: Transform {
                    translation: Vec3::new(self.exit_pos.x, -0.4, self.exit_pos.y),
                    ..Default::default()
                },
            };
            let _ = self
                .map
                .set_elem_pos(self.exit_pos, MapElement::Exit(exit_obj));
        }

        if self.exit_revealed {
            if self.players[0].is_colliding_with(self.exit_pos, 0.8) {
                println!("Level complete triggered!");
                if let Some(progress) = &mut self.campaign_progress {
                    let time_bonus = SCORE_RULES.time_bonus(progress.level_time);
                    let added = SCORE_RULES.level_clear.saturating_add(time_bonus);
                    progress.score = progress.score.saturating_add(added);
                }
                return GameTickResult::LevelComplete;
            }
        }

        GameTickResult::None
    }

    /// The tick function distributes behaviour between the
    /// [multiplayer tick function](GameState::mp_game_tick()) or the
    /// [campaign tick function](GameState::campaign_tick())
    pub fn tick(
        &mut self,
        delta_time: f32,
        inputs: &Vec<Input>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
        settings: &mut Settings,
    ) -> (Option<AppState>, u8) {
        if inputs.menu_back() == InputState::Pressed {
            return (Some(AppState::pause()), 0);
        }

        self.inputs_to_game_inputs(inputs);

        let result = match self.mode {
            GameMode::Multiplayer => {
                let mp_result = self.mp_game_tick(delta_time, resources, audio_manager);
                if let Some(winners) = mp_result.winners {
                    return (Some(AppState::multiplayer_end_screen(winners)), 1);
                }
                GameTickResult::None
            }
            GameMode::Campaign => self.campaign_tick(delta_time, resources, audio_manager),
        };

        match result {
            GameTickResult::LevelComplete => {
                if let Some(progress) = &self.campaign_progress {
                    (
                        Some(AppState::stage_clear(
                            settings,
                            progress.level,
                            progress.lives,
                            progress.score,
                            progress.difficulty,
                        )),
                        1,
                    )
                } else {
                    (None, 0)
                }
            }
            GameTickResult::GameOver => {
                let score = self
                    .campaign_progress
                    .as_ref()
                    .map(|progress| progress.score)
                    .unwrap_or(0);
                let difficulty = self
                    .campaign_progress
                    .as_ref()
                    .map(|progress| progress.difficulty)
                    .unwrap_or_default();
                (Some(AppState::game_over(score, difficulty)), 1)
            }
            GameTickResult::None => (None, 0),
        }
    }

    /// Copies all the players inputs to the gamestate. The extra layer is used so that bots and
    /// player inputs can come from the same source
    fn inputs_to_game_inputs(&mut self, inputs: &Vec<Input>) {
        for (i, input) in inputs.iter().enumerate() {
            if i < self.game_inputs.len() {
                self.game_inputs[i] = input.clone();
            }
        }
    }

    // TODO: doc
    pub fn create_ubo(&self, aspect_ratio: f32) -> GlobalUbo {
        let camera = {
            let mut camera = Camera::new();
            let clip = (self.map.width.max(self.map.height) as f32 / 2.0) * 1.15;
            camera.set_orthographic_projection(
                -clip * aspect_ratio,
                clip * aspect_ratio,
                -clip,
                clip,
                -clip,
                clip * 2.0,
            );
            camera.set_view_xyz(self.camera.translation, self.camera.rotation);
            camera
        };

        GlobalUbo {
            projection: camera.projection_matrix.to_cols_array_2d(),
            view: camera.view_matrix.to_cols_array_2d(),
            inverse_view: camera.inverse_view_matrix.to_cols_array_2d(),
            ambient_light_color: self.light.ambient_light_color.into(),
            direction_to_light: self.light.direction_to_light.to_array().into(),
            directional_light_color: self.light.directional_light_color.into(),
        }
    }

    /// Modifies a GameState's map for display purposes in game settings
    pub fn update_from_ui_settings(
        &mut self,
        settings: &UIGameSettings,
        resources: &Resources,
    ) -> Result<(), Box<dyn Error>> {
        let map_settings = settings.into_map_settings();

        match Map::new(map_settings, resources) {
            Some(map) => {
                self.camera = Transform {
                    translation: Vec3::new(map.width as f32 / 2.0, -1.0, map.height as f32 / 2.0),
                    scale: Vec3::ONE,
                    rotation: Vec3::new(-1.25, 0.0, 0.0),
                };
                self.map = map;
                Ok(())
            }
            None => Err("Map creation failed".into()),
        }
    }

    // TODO: remove or doc, currently unused
    pub fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(id as usize)
    }

    /// Gets an unmutable reference to the [GameState]'s [Map]
    pub fn get_map(&self) -> &Map {
        &self.map
    }
}
