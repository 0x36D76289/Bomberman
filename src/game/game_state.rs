use crate::app_state::AppState;
use crate::game::bomb::{Bomb, BombEvents, BombState, BombWorld};
use crate::game::bot::{Bot, BotLevel, BotView};
use crate::game::camera::Camera;
use crate::game::collision::Collision;
use crate::game::enemy::{Enemy, EnemyKind};
use crate::game::game_settings::GameSettings;
use crate::game::grid::{Tile, TileGrid};
use crate::game::lobby::LobbySettings;
use crate::game::map::map::{LEVEL_COUNT, LevelData, Map};
use crate::game::map::map_element::MapElement;
use crate::game::player::{Player, Powers};
use crate::game::powerup::{Curse, PowerUp, PowerUpType};
use crate::game::resources::Resources;
use crate::game::theme::Theme;
use crate::graphics::object::{AnimatedObject, Object};
use crate::graphics::transform::Transform;
use crate::graphics::{GlobalUbo, LightInfo, StateRenderInfo};
use crate::input::input::Input;
use crate::input::input_state::InputState;
use crate::input::input_vec::{GetOrDefault, MenuInput};
use crate::settings::save::{GameDifficulty, SaveState};
use crate::settings::settings::Settings;
use crate::{audio::AudioManager, audio::SoundEffect};
use glam::{Vec2, Vec3};
use rand::random_range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Multiplayer,
    Campaign,
}

#[derive(Debug, Clone, Copy)]
struct ScoreRules {
    breakable_destroyed: u32,
    powerup_pickup: u32,
    level_clear: u32,
    time_bonus_per_second: u32,
}

const SCORE_RULES: ScoreRules = ScoreRules {
    breakable_destroyed: 10,
    powerup_pickup: 50,
    level_clear: 1000,
    time_bonus_per_second: 10,
};

const INTRO_DURATION: f32 = 2.5;

#[derive(Default, Debug, Clone, Copy)]
struct TickEvents {
    breakables_destroyed: u32,
    enemies_score: u32,
    powerups_picked: u32,
    lives_picked: u32,
    explosions: u32,
    exit_hit: bool,
    last_picked: Option<PowerUpType>,
}

impl TickEvents {
    fn add_bomb_events(&mut self, events: BombEvents) {
        self.breakables_destroyed += events.breakables_destroyed;
        self.enemies_score += events.enemies_score;
        self.explosions += events.explosions;
        self.exit_hit |= events.exit_hit;
    }
}

#[derive(Debug, Clone, Copy)]
struct DifficultyPreset {
    enemy_speed_multiplier: f32,
    awareness: f32,
    extra_enemies: u32,
}

impl DifficultyPreset {
    fn for_level(level: u32, difficulty: GameDifficulty) -> Self {
        let level_index = (level.saturating_sub(1) * 7).div_ceil((LEVEL_COUNT - 1).max(1));
        let (speed, awareness, extra) = match difficulty {
            GameDifficulty::Easy => (0.85, 0.7, 0),
            GameDifficulty::Normal => (1.0, 0.9, level_index / 3),
            GameDifficulty::Hard => (1.1, 1.1, 1 + level_index / 2),
        };
        Self {
            enemy_speed_multiplier: (speed * (1.0 + level_index as f32 * 0.03)).clamp(0.6, 1.5),
            awareness: (awareness + level_index as f32 * 0.03).clamp(0.5, 1.4),
            extra_enemies: extra,
        }
    }

    fn pick_kind(level: u32) -> EnemyKind {
        let pool: &[EnemyKind] = match level {
            0..=1 => &[EnemyKind::Balloom, EnemyKind::Balloom, EnemyKind::Onil],
            2 => &[EnemyKind::Balloom, EnemyKind::Onil, EnemyKind::Doll],
            3 => &[EnemyKind::Onil, EnemyKind::Onil, EnemyKind::Doll],
            4 => &[EnemyKind::Onil, EnemyKind::Doll, EnemyKind::Minvo],
            _ => &[EnemyKind::Doll, EnemyKind::Minvo, EnemyKind::Minvo],
        };
        pool[random_range(0..pool.len())]
    }

    fn strongest_kind(level: u32) -> EnemyKind {
        match level {
            0..=1 => EnemyKind::Onil,
            2..=3 => EnemyKind::Doll,
            _ => EnemyKind::Minvo,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CampaignProgress {
    pub save: SaveState,
    pub score: u32,
    pub lives: u32,
    pub level_time: f32,
    pub time_limit: f32,
    pub name: String,
    hurry: bool,
    exit_open: bool,
    exit_visible: bool,
    pub clear_stats: ClearStats,
    last_pickup: Option<(PowerUpType, f32)>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClearStats {
    pub stage_points: u32,
    pub clear_bonus: u32,
    pub time_bonus: u32,
    pub total: u32,
}

#[derive(Debug, Clone)]
pub struct Banner {
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone)]
pub enum HudInfo {
    Campaign {
        lives: u32,
        powers: Powers,
        pickup: Option<&'static str>,
        level: u32,
        level_count: u32,
        world: &'static str,
        time_left: f32,
        hurry: bool,
        score: u32,
    },
    Multiplayer(
        Vec<(u32, bool, bool, Option<&'static str>)>,
        Option<Vec<u32>>,
    ),
}

impl CampaignProgress {
    pub fn time_left(&self) -> f32 {
        (self.time_limit - self.level_time).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Phase {
    Intro(f32),
    Playing,
    Dying(f32),
    Over,
}

#[derive(Debug, Clone)]
pub enum GameTickResult {
    None,
    Died,
    LevelComplete,
}

#[derive(Debug, Clone)]
pub struct GameState {
    mode: GameMode,
    campaign_progress: Option<CampaignProgress>,
    phase: Phase,
    players: Vec<Player>,
    bots: Vec<Option<Bot>>,
    enemies: Vec<Enemy>,
    exit_pos: Vec2,
    player_spawn: Vec2,
    game_inputs: Vec<Input>,
    bombs: Vec<Bomb>,
    power_ups: Vec<PowerUp>,
    map: Map,
    camera: Transform,
    light: LightInfo,
    shake: f32,
    shake_time: f32,
    alive_players: Vec<u32>,
    pub render_info: StateRenderInfo,
}

fn camera_for(map: &Map) -> Transform {
    Transform {
        translation: Vec3::new(map.width as f32 / 2.0, -1.0, map.height as f32 / 2.0),
        scale: Vec3::ONE,
        rotation: Vec3::new(-1.25, 0.0, 0.0),
    }
}

impl GameState {
    fn with_map(
        mode: GameMode,
        map: Map,
        players: Vec<Player>,
        nb_humans: u32,
        bot_level: BotLevel,
    ) -> Self {
        Self {
            mode,
            campaign_progress: None,
            phase: Phase::Playing,
            alive_players: players.iter().map(|player| player.id).collect(),
            game_inputs: vec![Input::default(); players.len()],
            bots: players
                .iter()
                .map(|p| (p.id >= nb_humans).then(|| Bot::new(bot_level)))
                .collect(),
            players,
            enemies: Vec::new(),
            exit_pos: Vec2::ZERO,
            player_spawn: Vec2::ZERO,
            bombs: Vec::new(),
            power_ups: Vec::new(),
            camera: camera_for(&map),
            light: map.theme.light(),
            shake: 0.0,
            shake_time: 0.0,
            map,
            render_info: StateRenderInfo {
                drawn_first: true,
                ..Default::default()
            },
        }
    }

    fn scenery(map: Map) -> Self {
        Self::with_map(
            GameMode::Multiplayer,
            map,
            Vec::new(),
            0,
            BotLevel::default(),
        )
    }

    pub fn backdrop(resources: &Resources, theme: Theme) -> Self {
        Self::scenery(Map::backdrop(resources, theme))
    }

    pub fn new_campaign(save: SaveState) -> Option<Self> {
        let resources = crate::resources();
        let LevelData {
            map,
            player_spawn,
            enemy_spawns,
            exit_pos,
            name,
            time_limit,
        } = Map::from_file(
            save.level,
            save.random_layout.then_some(save.seed),
            resources,
        )?;

        let preset = DifficultyPreset::for_level(save.level, save.difficulty);
        let mut spawns = enemy_spawns;
        let mut avoid: Vec<Vec2> = vec![player_spawn];
        avoid.extend(spawns.iter().map(|(pos, _)| *pos));
        for _ in 0..preset.extra_enemies {
            if let Some(pos) = map.find_random_empty(&avoid, 4.0, 60) {
                spawns.push((pos, None));
                avoid.push(pos);
            }
        }

        let mut state = Self::with_map(
            GameMode::Campaign,
            map,
            vec![Player::new(
                0,
                player_spawn,
                super::direction::Direction::Down,
                resources,
                true,
                save.powers,
            )],
            1,
            BotLevel::default(),
        );
        state.enemies = spawns
            .iter()
            .map(|(pos, kind)| {
                Enemy::new(
                    *pos,
                    kind.unwrap_or_else(|| DifficultyPreset::pick_kind(save.level)),
                    preset.enemy_speed_multiplier,
                    preset.awareness,
                    resources,
                )
            })
            .collect();
        state.exit_pos = exit_pos;
        state.player_spawn = player_spawn;
        state.phase = Phase::Intro(INTRO_DURATION);
        state.campaign_progress = Some(CampaignProgress {
            save,
            score: save.score,
            lives: save.lives,
            level_time: 0.0,
            time_limit,
            name,
            ..Default::default()
        });
        Some(state)
    }

    pub fn lobby_preview(lobby: &LobbySettings, resources: &Resources) -> Self {
        Self::scenery(
            Map::new(lobby.map_settings(), resources)
                .unwrap_or_else(|| Map::backdrop(resources, lobby.theme)),
        )
    }

    pub fn new_multiplayer_from_map(
        resources: &Resources,
        settings: GameSettings,
        map: Map,
    ) -> Self {
        let players = Self::create_players(
            &map,
            resources,
            settings.nb_humans,
            settings.nb_bots,
            settings.teams.as_deref(),
        );
        Self::with_map(
            GameMode::Multiplayer,
            map,
            players,
            settings.nb_humans,
            settings.bot_level,
        )
    }

    fn create_players(
        map: &Map,
        resources: &Resources,
        nb_humans: u32,
        nb_bots: u32,
        teams: Option<&[u32]>,
    ) -> Vec<Player> {
        let count = map.spawns.len();
        (0..(nb_humans + nb_bots) as usize)
            .filter(|id| match teams {
                Some(t) => *id < t.len() && count > 0,
                None => *id < count,
            })
            .map(|id| {
                let spawn = &map.spawns[match teams {
                    Some(t) => {
                        (t[id] as usize
                            + t[..id].iter().filter(|team| **team == t[id]).count()
                                * (count / 2).max(1))
                            % count
                    }
                    None => id,
                }];
                let mut player = Player::new(
                    id as u32,
                    spawn.position(),
                    spawn.direction,
                    resources,
                    (id as u32) < nb_humans,
                    Powers::default(),
                );
                player.team = teams.map(|t| t[id]);
                player.base_color = player_color(id, teams);
                if let Some(obj) = &mut player.object {
                    obj.color = player.base_color;
                }
                player
            })
            .collect()
    }

    pub fn objects_to_render(&self) -> impl Iterator<Item = &Object> {
        self.map
            .iter()
            .filter_map(|el| match el {
                MapElement::Empty => None,
                MapElement::Breakable(obj) => Some(obj),
                MapElement::Unbreakable(obj) => Some(obj),
                MapElement::Exit(obj) => Some(obj),
            })
            .chain(std::iter::once(&self.map.floor))
            .chain(self.map.decor.iter())
            .chain(self.bombs.iter().flat_map(|b| b.objects()))
            .chain(self.power_ups.iter().map(|p| &p.object))
    }

    pub fn animated_objects_to_render(&self) -> impl Iterator<Item = &AnimatedObject> {
        self.players
            .iter()
            .filter_map(|p| p.object.as_ref())
            .chain(self.enemies.iter().filter_map(|e| e.object.as_ref()))
    }

    fn simulate(
        &mut self,
        delta: f32,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> TickEvents {
        let mut events = TickEvents::default();
        for i in 0..self.bombs.len() {
            let others: Vec<&Bomb> = self
                .bombs
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != i)
                .map(|(_, bomb)| bomb)
                .collect();
            let bombs_pos: Vec<Vec2> = others.iter().map(|bomb| bomb.position).collect();
            let burning: Vec<Tile> = others
                .iter()
                .flat_map(|bomb| bomb.burning_tiles())
                .collect();

            events.add_bomb_events(self.bombs[i].tick(
                delta,
                &mut BombWorld {
                    players: &mut self.players,
                    enemies: &mut self.enemies,
                    map: &mut self.map,
                    power_ups: &mut self.power_ups,
                    resources,
                    audio_manager,
                    bombs_pos: &bombs_pos,
                    burning: &burning,
                },
            ));
        }
        self.shake = (self.shake - 1.2 * delta).max(0.0);
        if events.explosions > 0 {
            self.shake = (self.shake + 0.12 * events.explosions as f32).min(0.35);
        }
        self.shake_time = if self.shake > 0.0 {
            self.shake_time + delta
        } else {
            0.0
        };
        for i in 0..self.bombs.len() {
            if self.bombs[i].state != BombState::Exploding {
                continue;
            }
            self.bombs[i].clone().chain_react(&mut self.bombs);
        }
        self.bombs.retain(|bomb| !bomb.despawn);
        for bomb in &mut self.bombs {
            if !self
                .players
                .get(bomb.owner_id as usize)
                .is_some_and(|p| p.alive && p.powers.remote)
            {
                bomb.remote = false;
            }
        }

        for powerup in &mut self.power_ups {
            if let Some(kind) = powerup.tick(&mut self.players, audio_manager) {
                match kind {
                    PowerUpType::ExtraLife => events.lives_picked += 1,
                    PowerUpType::Skull => (),
                    _ => events.powerups_picked += 1,
                }
                events.last_picked = Some(kind);
            }
        }
        self.power_ups.retain(|powerup| !powerup.despawn);

        for player in &mut self.players {
            player.tick_status(delta);
        }
        self.spread_curses();

        for (i, player) in self.players.iter().enumerate() {
            if !player.alive
                || !player.powers.remote
                || self.game_inputs.get_or_default(i).detonate() != InputState::Pressed
            {
                continue;
            }
            if let Some(bomb) = self
                .bombs
                .iter_mut()
                .find(|b| b.remote && b.owner_id == player.id && b.state != BombState::Exploding)
            {
                bomb.remote = false;
                bomb.timer = 0.0;
            }
        }

        let mut planted = vec![false; self.players.len()];
        for (i, player) in self.players.iter_mut().enumerate() {
            let wants = self.game_inputs.get_or_default(i).bomb() == InputState::Pressed;
            if !player.alive || !(wants || player.cursed_with(Curse::Diarrhea)) {
                continue;
            }
            match player.create_bomb(resources, &self.bombs, &self.map) {
                Some(bomb) => {
                    audio_manager.play_sound_effect(SoundEffect::PutBomb);
                    planted[i] = true;
                    self.bombs.push(bomb)
                }
                None if player.is_human && wants => {
                    audio_manager.play_sound_effect(SoundEffect::NoBomb)
                }
                None => (),
            }
        }

        let sliding = |bombs: &[Bomb]| {
            bombs
                .iter()
                .filter(|b| matches!(b.state, BombState::Sliding(_)))
                .count()
        };
        let sliding_before = sliding(&self.bombs);
        for (i, planted) in planted.into_iter().enumerate() {
            let occupied: Vec<Tile> = self
                .players
                .iter()
                .enumerate()
                .filter(|(j, other)| *j != i && other.alive)
                .flat_map(|(_, other)| other.footprint())
                .collect();
            let player = &mut self.players[i];
            player.player_move(
                self.game_inputs.get_or_default(i),
                delta,
                &self.map,
                &mut self.bombs,
                &occupied,
            );
            player.animate(resources, delta, planted);
        }
        if sliding(&self.bombs) > sliding_before {
            audio_manager.play_sound_effect(SoundEffect::Kick);
        }
        events
    }

    fn spread_curses(&mut self) {
        for i in 0..self.players.len() {
            let Some(curse) = self.players[i].curse.filter(|_| self.players[i].alive) else {
                continue;
            };
            let from = self.players[i].position;
            for other in self.players.iter_mut() {
                if other.alive && other.curse.is_none() && other.position.distance(from) < 1.05 {
                    other.curse = Some(curse);
                }
            }
        }
    }

    fn teams(&self) -> Option<Vec<u32>> {
        self.players
            .iter()
            .map(|player| player.team)
            .collect::<Option<Vec<u32>>>()
            .filter(|teams| !teams.is_empty())
    }

    fn create_mp_ret(&mut self) -> Option<Vec<u32>> {
        let alive: Vec<&Player> = self.players.iter().filter(|player| player.alive).collect();
        let alive_players: Vec<u32> = alive.iter().map(|player| player.id).collect();

        if alive_players.is_empty() {
            return Some(self.alive_players.clone());
        }
        if match alive[0].team {
            Some(team) => alive.iter().all(|player| player.team == Some(team)),
            None => alive.len() == 1,
        } {
            return Some(alive_players);
        }
        self.alive_players = alive_players;
        None
    }

    fn spawn_reinforcements(
        &mut self,
        count: usize,
        kind: EnemyKind,
        speed: f32,
        resources: &Resources,
    ) {
        let preset = DifficultyPreset::for_level(
            self.campaign_progress.as_ref().map_or(1, |p| p.save.level),
            self.campaign_progress
                .as_ref()
                .map_or(GameDifficulty::Normal, |p| p.save.difficulty),
        );
        let mut avoid = vec![self.players[0].position];
        for _ in 0..count {
            let Some(pos) = self.map.find_random_empty(&avoid, 5.0, 80) else {
                continue;
            };
            avoid.push(pos);
            self.enemies.push(Enemy::new(
                pos,
                kind,
                preset.enemy_speed_multiplier * speed,
                preset.awareness,
                resources,
            ));
        }
    }

    fn campaign_tick(
        &mut self,
        delta: f32,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> GameTickResult {
        match self.phase {
            Phase::Intro(timer) => {
                if timer == INTRO_DURATION {
                    audio_manager.play_sound_effect(SoundEffect::LevelStart);
                }
                self.phase = if timer - delta <= 0.0 {
                    Phase::Playing
                } else {
                    Phase::Intro(timer - delta)
                };
                return GameTickResult::None;
            }
            Phase::Dying(timer) => {
                if timer - delta <= 0.0 {
                    if !self.respawn(resources) {
                        return GameTickResult::Died;
                    }
                    return GameTickResult::None;
                }
                self.phase = Phase::Dying(timer - delta);
                self.game_inputs.fill(Input::default());
            }
            Phase::Playing => {}
            Phase::Over => return GameTickResult::None,
        }

        let mut hurry = false;
        if let Some(progress) = &mut self.campaign_progress {
            progress.level_time += delta;
            if !progress.hurry && progress.time_left() <= 0.0 {
                progress.hurry = true;
                hurry = true;
            }
        }
        if hurry {
            audio_manager.play_sound_effect(SoundEffect::Alarm);
            self.spawn_reinforcements(4, EnemyKind::Minvo, 1.2, resources);
        }

        let player_pos = self.players.first().filter(|p| p.alive).map(|p| p.position);
        let grid = TileGrid::capture(&self.map, &self.bombs);
        let blast = grid.blast_times();
        for i in 0..self.enemies.len() {
            let occupied: Vec<Tile> = self
                .enemies
                .iter()
                .enumerate()
                .filter(|(j, other)| *j != i && other.alive)
                .flat_map(|(_, other)| other.tiles())
                .collect();
            let enemy = &mut self.enemies[i];
            enemy.tick(delta, &grid, &blast, player_pos, &occupied);
            enemy.animate(resources, delta);
            if self.players[0].alive
                && self.players[0].invincible <= 0.0
                && enemy.touches(self.players[0].position)
            {
                self.players[0].kill();
                audio_manager.play_sound_effect(SoundEffect::PlayerDeath);
            }
        }

        let events = self.simulate(delta, resources, audio_manager);
        if let Some(progress) = &mut self.campaign_progress {
            progress.score += events.enemies_score
                + events.breakables_destroyed * SCORE_RULES.breakable_destroyed
                + events.powerups_picked * SCORE_RULES.powerup_pickup;
            progress.lives += events.lives_picked;
            progress.last_pickup = match (events.last_picked, progress.last_pickup) {
                (Some(kind), _) => Some((kind, 2.0)),
                (None, Some((kind, left))) if left > delta => Some((kind, left - delta)),
                _ => None,
            };
        }
        if events.exit_hit {
            self.spawn_reinforcements(
                4,
                DifficultyPreset::strongest_kind(
                    self.campaign_progress.as_ref().map_or(1, |p| p.save.level),
                ),
                1.0,
                resources,
            );
        }
        self.enemies.retain(|e| e.alive);

        if self.phase == Phase::Playing && !self.players[0].alive {
            self.phase = Phase::Dying(2.5);
        }

        self.update_exit(resources, audio_manager);

        if self
            .campaign_progress
            .as_ref()
            .is_some_and(|p| p.exit_open && p.exit_visible)
            && self.phase == Phase::Playing
            && self.players[0].is_colliding_with(self.exit_pos, 0.3)
        {
            if let Some(progress) = &mut self.campaign_progress {
                let time_bonus = progress.time_left() as u32 * SCORE_RULES.time_bonus_per_second;
                let stage_points = progress.score.saturating_sub(progress.save.score);
                progress.score += SCORE_RULES.level_clear + time_bonus;
                progress.clear_stats = ClearStats {
                    stage_points,
                    clear_bonus: SCORE_RULES.level_clear,
                    time_bonus,
                    total: progress.score,
                };
            }
            return GameTickResult::LevelComplete;
        }

        GameTickResult::None
    }

    fn respawn(&mut self, resources: &Resources) -> bool {
        let Some(progress) = &mut self.campaign_progress else {
            return false;
        };
        if progress.lives <= 1 {
            return false;
        }
        progress.lives -= 1;
        let player = &mut self.players[0];
        player.respawn(
            resources,
            match self.map.get_elem_pos(player.position) {
                MapElement::Empty | MapElement::Exit(_) => player.position,
                _ => self.player_spawn,
            },
            3.0,
        );
        self.phase = Phase::Playing;
        true
    }

    fn update_exit(&mut self, resources: &Resources, audio_manager: &mut AudioManager) {
        let Some(progress) = &mut self.campaign_progress else {
            return;
        };
        if !progress.exit_visible && *self.map.get_elem_pos(self.exit_pos) == MapElement::Empty {
            progress.exit_visible = true;
            self.map.set_elem_pos(
                self.exit_pos,
                MapElement::Exit(Map::create_exit(resources, self.exit_pos)),
            );
        }
        if !progress.exit_open && self.enemies.is_empty() {
            progress.exit_open = true;
            audio_manager.play_sound_effect(SoundEffect::DoorOpen);
        }
        if let Some(MapElement::Exit(obj)) = self.map.get_elem_mut_pos(self.exit_pos) {
            obj.color = if progress.exit_open {
                Vec3::ONE
            } else {
                Vec3::splat(0.35)
            };
        }
    }

    pub fn tick(
        &mut self,
        delta_time: f32,
        inputs: &Vec<Input>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
        settings: &mut Settings,
    ) -> (Option<AppState>, u8) {
        if inputs.menu_back() == InputState::Pressed {
            audio_manager.pause_background_music();
            return (Some(AppState::pause(self.restart_info())), 0);
        }

        self.update_game_inputs(inputs, delta_time);

        match self.mode {
            GameMode::Multiplayer => {
                self.simulate(delta_time, resources, audio_manager);
                if let Some(winners) = self.create_mp_ret() {
                    audio_manager.play_sound_effect(SoundEffect::Victory);
                    return (
                        Some(AppState::multiplayer_end_screen(
                            winners,
                            self.restart_info(),
                        )),
                        0,
                    );
                }
                (None, 0)
            }
            GameMode::Campaign => {
                let result = self.campaign_tick(delta_time, resources, audio_manager);
                let Some(progress) = &self.campaign_progress else {
                    return (None, 0);
                };
                let mut run = progress.save;
                run.score = progress.score;
                run.lives = progress.lives;
                run.powers = self.players[0].powers;
                match result {
                    GameTickResult::None => (None, 0),
                    GameTickResult::LevelComplete => {
                        audio_manager.play_sound_effect(SoundEffect::StageClear);
                        (
                            Some(AppState::stage_clear(
                                settings,
                                run,
                                progress.name.clone(),
                                progress.clear_stats,
                            )),
                            0,
                        )
                    }
                    GameTickResult::Died => {
                        self.phase = Phase::Over;
                        audio_manager.play_sound_effect(SoundEffect::GameOver);
                        audio_manager.stop_background_music();
                        (Some(AppState::game_over(run)), 0)
                    }
                }
            }
        }
    }

    pub fn restart_info(&self) -> RestartInfo {
        match &self.campaign_progress {
            Some(progress) => RestartInfo::Campaign(progress.save),
            None => RestartInfo::Multiplayer {
                map: Box::new(self.map.clone()),
                settings: GameSettings {
                    nb_humans: self.bots.iter().filter(|b| b.is_none()).count() as u32,
                    nb_bots: self.bots.iter().filter(|b| b.is_some()).count() as u32,
                    bot_level: self
                        .bots
                        .iter()
                        .flatten()
                        .next()
                        .map_or(BotLevel::default(), Bot::level),
                    teams: self.teams(),
                },
            },
        }
    }

    fn update_game_inputs(&mut self, inputs: &[Input], delta: f32) {
        let grid = if self.bots.iter().any(|b| b.is_some()) {
            TileGrid::capture(&self.map, &self.bombs)
        } else {
            TileGrid::new(0, 0)
        };
        let view = BotView {
            grid: &grid,
            blast: &grid.blast_times(),
            players: &self.players,
            power_ups: &self.power_ups,
        };
        let mut human = 0;
        for (i, bot) in self.bots.iter_mut().enumerate() {
            self.game_inputs[i] = match bot {
                Some(bot) => bot.think(&self.players[i], &view, delta),
                None => {
                    human += 1;
                    inputs.get(human - 1).copied().unwrap_or_default()
                }
            };
        }
        if matches!(self.phase, Phase::Intro(_)) {
            self.game_inputs.fill(Input::default());
        }
    }

    pub fn create_ubo(&self, aspect_ratio: f32) -> GlobalUbo {
        let camera = {
            let mut camera = Camera::new();
            let clip = (self.map.height as f32 / 2.0 * self.camera.rotation.x.abs().sin() + 0.9)
                .max((self.map.width as f32 / 2.0 + 0.4) / aspect_ratio)
                * 1.12;
            camera.set_orthographic_projection(
                -clip * aspect_ratio,
                clip * aspect_ratio,
                -clip,
                clip,
                -clip * 3.0,
                clip * 3.0,
            );
            camera.set_view_xyz(
                self.camera.translation
                    + Vec3::new(0.0, 0.0, clip * 0.1)
                    + self.shake
                        * Vec3::new(
                            ((self.shake_time * 20.0).floor() * 2.1).sin().signum(),
                            0.0,
                            ((self.shake_time * 20.0).floor() * 1.7 + 1.3)
                                .sin()
                                .signum(),
                        ),
                self.camera.rotation,
            );
            camera
        };

        GlobalUbo {
            projection: camera.projection_matrix.to_cols_array_2d(),
            view: camera.view_matrix.to_cols_array_2d(),
            ambient_light_color: self.light.ambient_light_color.into(),
            direction_to_light: self.light.direction_to_light.to_array().into(),
            directional_light_color: self.light.directional_light_color.into(),
        }
    }

    pub fn update_from_lobby(&mut self, lobby: &LobbySettings, resources: &Resources) {
        if let Some(map) = Map::new(lobby.map_settings(), resources) {
            self.camera = camera_for(&map);
            self.light = map.theme.light();
            self.map = map;
        }
    }

    pub fn get_map(&self) -> &Map {
        &self.map
    }

    pub fn hud_info(&self) -> HudInfo {
        match &self.campaign_progress {
            Some(progress) => HudInfo::Campaign {
                lives: progress.lives,
                powers: self.players[0].powers,
                pickup: progress.last_pickup.map(|(kind, _)| kind.label()),
                level: progress.save.level,
                level_count: LEVEL_COUNT,
                world: self.map.theme.display_name(),
                time_left: progress.time_left(),
                hurry: progress.hurry,
                score: progress.score,
            },
            None => HudInfo::Multiplayer(
                self.players
                    .iter()
                    .map(|p| {
                        (
                            p.id,
                            p.is_human,
                            p.alive,
                            p.curse.filter(|_| p.alive).map(|(c, _)| c.label()),
                        )
                    })
                    .collect(),
                self.teams(),
            ),
        }
    }

    pub fn banner(&self) -> Option<Banner> {
        let progress = self.campaign_progress.as_ref()?;
        match self.phase {
            Phase::Intro(_) => Some(Banner {
                title: format!("NIVEAU {}", progress.save.level),
                subtitle: format!("{} - {}", self.map.theme.display_name(), progress.name),
            }),
            Phase::Dying(_) => Some(Banner {
                title: "AÏE !".to_string(),
                subtitle: match progress.lives.saturating_sub(1) {
                    0 => "Plus aucune vie".to_string(),
                    1 => "Plus qu'une vie".to_string(),
                    n => format!("Encore {n} vies"),
                },
            }),
            Phase::Playing | Phase::Over => None,
        }
    }

    pub fn theme(&self) -> Theme {
        self.map.theme
    }
}

#[derive(Debug, Clone)]
pub enum RestartInfo {
    Campaign(SaveState),
    Multiplayer {
        map: Box<Map>,
        settings: GameSettings,
    },
}

impl RestartInfo {
    pub fn restart(&self, resources: &Resources) -> Option<GameState> {
        match self {
            RestartInfo::Campaign(save) => GameState::new_campaign(*save),
            RestartInfo::Multiplayer { map, settings } => Some(
                GameState::new_multiplayer_from_map(resources, settings.clone(), (**map).clone()),
            ),
        }
    }
}

pub fn player_ui_color(id: usize, teams: Option<&[u32]>) -> glam::Vec4 {
    player_color(id, teams).extend(1.0)
}

pub fn player_color(id: usize, teams: Option<&[u32]>) -> Vec3 {
    match teams.and_then(|t| t.get(id)) {
        Some(team) => team_color(*team as usize),
        None => team_color(id),
    }
}

pub fn team_color(id: usize) -> Vec3 {
    const COLORS: [Vec3; 8] = [
        Vec3::new(0.99, 0.99, 0.99),
        Vec3::new(0.97, 0.22, 0.0),
        Vec3::new(0.24, 0.47, 0.99),
        Vec3::new(0.0, 0.85, 0.1),
        Vec3::new(0.97, 0.85, 0.0),
        Vec3::new(0.97, 0.47, 0.97),
        Vec3::new(0.0, 0.91, 0.85),
        Vec3::new(0.99, 0.6, 0.27),
    ];
    COLORS[id % COLORS.len()]
}
