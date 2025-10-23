use crate::app_state::AppState;
use crate::game::bomb::{Bomb, BombState};
use crate::game::camera::Camera;
use crate::game::collision::Collision;
use crate::game::enemy::Enemy;
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
use crate::settings::settings::Settings;
use crate::ui::pages::game_settings::UIGameSettings;
use crate::{audio::AudioManager, audio::SoundEffect};
use glam::{Vec2, Vec3, Vec4};
use std::error::Error;
use std::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Multiplayer,
    Campaign,
}

#[derive(Debug, Clone, Default)]
pub struct CampaignProgress {
    pub level: u32,
    pub lives: u32,
    pub score: u32,
}

#[derive(Debug, Clone)]
pub enum GameTickResult {
    None,
    GameOver,
    LevelComplete,
}

#[derive(Debug, Clone)]
pub struct GameState {
    mode: GameMode,
    campaign_progress: Option<CampaignProgress>,
    players: Vec<Player>,
    enemies: Vec<Enemy>,
    exit_pos: Vec2,
    exit_revealed: bool,
    game_inputs: Vec<Input>,
    nb_humans: u32,
    bombs: Vec<Bomb>,
    power_ups: Vec<PowerUp>,
    map: Map,
    camera: Transform,
    light: LightInfo,
    alive_players: Vec<u32>,
    pub render_info: StateRenderInfo,
}

impl GameState {
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

    pub fn new_campaign(level: u32, lives: u32) -> Option<Self> {
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

        let mut enemies = Vec::new();
        for (i, spawn) in enemy_spawns.iter().enumerate() {
            enemies.push(Enemy::new(i as u32, *spawn, resources_to_load_map));
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
                score: 0,
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

    fn mp_game_tick(
        &mut self,
        delta: f32,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> Option<Vec<u32>> {
        // tick bombs
        for i in 0..self.bombs.len() {
            let bombs_pos = self
                .bombs
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != i)
                .map(|(_, bomb)| bomb.position)
                .collect::<Vec<_>>();

            self.bombs[i].tick(
                delta,
                &mut self.players,
                &mut self.enemies,
                &mut self.map,
                &mut self.power_ups,
                resources,
                audio_manager,
                &bombs_pos,
            );
        }
        for i in 0..self.bombs.len() {
            if self.bombs[i].state != BombState::Exploding {
                continue;
            }
            self.bombs[i].clone().chain_react(&mut self.bombs);
        }
        self.bombs.retain(|bomb| !bomb.despawn);
        for powerup in &mut self.power_ups {
            powerup.tick(&mut self.players, audio_manager);
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
        self.create_mp_ret()
    }

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

        // Tick enemies
        for i in 0..self.enemies.len() {
            let (left, right) = self.enemies.split_at_mut(i);
            if let Some((current, right)) = right.split_first_mut() {
                let other_enemies: Vec<_> = left.iter().chain(right.iter()).cloned().collect();
                current.tick(delta, &self.map, &self.bombs, &other_enemies);
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
        self.mp_game_tick(delta, resources, audio_manager);

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
                return GameTickResult::LevelComplete;
            }
        }

        GameTickResult::None
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
            return (Some(AppState::pause()), 0);
        }

        self.inputs_to_game_inputs(inputs);

        let result = match self.mode {
            GameMode::Multiplayer => {
                if let Some(winners) = self.mp_game_tick(delta_time, resources, audio_manager) {
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
                        )),
                        1,
                    )
                } else {
                    (None, 0)
                }
            }
            GameTickResult::GameOver => (Some(AppState::game_over()), 1),
            GameTickResult::None => (None, 0),
        }
    }

    fn inputs_to_game_inputs(&mut self, inputs: &Vec<Input>) {
        for (i, input) in inputs.iter().enumerate() {
            if i < self.game_inputs.len() {
                self.game_inputs[i] = input.clone();
            }
        }
    }

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

    pub fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(id as usize)
    }

    pub fn get_map(&self) -> &Map {
        &self.map
    }
}
