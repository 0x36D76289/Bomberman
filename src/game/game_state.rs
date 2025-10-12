use crate::app_state::AppState;
use crate::game::ai::cpu::CPU;
use crate::game::camera::Camera;

use crate::game::ai::ai::AI;
use crate::game::ai::zone;
use crate::game::bomb::{Bomb, BombState};
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
use crate::graphics::renderer::RENDER_RES_RATIO;
use crate::graphics::transform::Transform;
use crate::graphics::{GamePush, GlobalUbo, LightInfo, Renderer, Vulkan};
use crate::input::input::Input;
use crate::input::input_state::InputState;
use crate::input::input_vec::{GetOrDefault, MenuInput};
use crate::ui::UiState;
use crate::{audio::AudioManager, audio::SoundEffect};
use glam::{Vec2, Vec3, Vec4, bool};
use rand::random_range;
use std::error::Error;
use std::sync::Arc;
use std::vec::Vec;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferInheritanceInfo,
        CommandBufferInheritanceRenderPassType, CommandBufferInheritanceRenderingInfo,
        CommandBufferUsage, SecondaryAutoCommandBuffer,
    },
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    format::Format,
    pipeline::{Pipeline, PipelineBindPoint, graphics::viewport::Viewport},
};

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
    cpus: Vec<CPU>,
    game_inputs: Vec<Input>,
    nb_humans: usize,
    bombs: Vec<Bomb>,
    power_ups: Vec<PowerUp>,
    map: Map,
    camera: Transform,
    light: LightInfo,
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
        let players = Self::create_players(&map, &resources, &nb_humans);
        let game_inputs = vec![Input::default(); players.len()];
        let mut cpus: Vec<CPU> = (nb_humans..players.len()).map(|id| CPU::new(id)).collect();
        for cpu in &mut cpus {
            cpu.update_zone(players[cpu.id].position, &players, &map);
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

        Ok(Self {
            mode: GameMode::Multiplayer,
            campaign_progress: None,
            players,
            cpus,
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
        let mut cpus = [].to_vec();
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

        Some(Self {
            mode: GameMode::Campaign,
            campaign_progress: Some(CampaignProgress {
                level,
                lives,
                score: 0,
            }),
            players,
            cpus,
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
        })
    }

    fn create_players(map: &Map, resources: &Resources, nb_humans: &usize) -> Vec<Player> {
        let mut players = Vec::<Player>::new();
        let mut id: usize = 0;
        for spawn in map.spawns.clone() {
            players.push(Player::new(
                id as u32,
                Vec2 {
                    x: spawn.x as f32 + 0.5,
                    y: spawn.y as f32 + 0.5,
                },
                spawn.direction,
                resources,
                id < *nb_humans,
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
    ) {
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
            AI::update_zone(
                self.bombs[i].position,
                &mut self.cpus,
                &self.players,
                &self.map,
            );
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
    ) -> (Option<AppState>, u8) {
        if inputs.menu_back() == InputState::Pressed {
            return (Some(AppState::Ui(UiState::pause())), 0);
        }
        self.update_human_inputs(inputs);

        let result = match self.mode {
            GameMode::Multiplayer => {
                self.update_cpu_inputs();
                self.mp_game_tick(delta_time, resources, audio_manager);
                GameTickResult::None
            }
            GameMode::Campaign => self.campaign_tick(delta_time, resources, audio_manager),
        };

        match result {
            GameTickResult::LevelComplete => {
                if let Some(progress) = &self.campaign_progress {
                    (
                        Some(AppState::Ui(UiState::stage_clear(
                            progress.level,
                            progress.lives,
                        ))),
                        1,
                    )
                } else {
                    (None, 0)
                }
            }
            GameTickResult::GameOver => (Some(AppState::Ui(UiState::game_over())), 1),
            GameTickResult::None => (None, 0),
        }
    }

    fn update_cpu_inputs(&mut self) {
        self.cpus.iter_mut().enumerate().for_each(|(i, cpu)| {
            self.game_inputs[self.nb_humans as usize + i] =
                cpu.get_input(&self.power_ups, &self.players, &self.map)
        });
    }
    // Put the inputs read into game inputs
    fn update_human_inputs(&mut self, inputs: &Vec<Input>) {
        for (i, input) in inputs.iter().enumerate() {
            if i < self.game_inputs.len() {
                self.game_inputs[i] = input.clone();
            }
        }
    }

    pub fn render(
        &self,
        vulkan: &Vulkan,
        renderer: &Renderer,
        resources: &Resources,
    ) -> Arc<SecondaryAutoCommandBuffer> {
        let pipeline = match renderer.game_pipeline.as_ref() {
            Some(pipeline) => pipeline.clone(),
            None => panic!(
                "Called render on a GameState object but the game_pipeline is not initialized in the renderer"
            ),
        };

        let window_size: [u32; 2] = renderer.window_size();
        let game_resolution = [
            window_size[0] / RENDER_RES_RATIO[0],
            window_size[1] / RENDER_RES_RATIO[1],
        ];

        let aspect_ratio = window_size[0] as f32 / window_size[1] as f32;
        let mut camera = Camera::new();
        let mut clipping = self.map.width.max(self.map.height) as f32 / 2.0;
        clipping *= 1.15;
        camera.set_orthographic_projection(
            -clipping * aspect_ratio,
            clipping * aspect_ratio,
            -clipping,
            clipping,
            -clipping,
            clipping,
        );
        camera.set_view_xyz(self.camera.translation, self.camera.rotation);

        let global_ubo = GlobalUbo {
            projection: camera.projection_matrix.to_cols_array_2d(),
            view: camera.view_matrix.to_cols_array_2d(),
            inverse_view: camera.inverse_view_matrix.to_cols_array_2d(),
            ambient_light_color: self.light.ambient_light_color.into(),
            direction_to_light: self.light.direction_to_light.to_array().into(),
            directional_light_color: self.light.directional_light_color.into(),
        };

        let format = renderer.rcx().swapchain.image_format();

        let inheritance_rendering_info = CommandBufferInheritanceRenderingInfo {
            color_attachment_formats: vec![Some(format)],
            depth_attachment_format: Some(Format::D32_SFLOAT),
            ..Default::default()
        };

        let mut secondary_builder = AutoCommandBufferBuilder::secondary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(CommandBufferInheritanceRenderPassType::BeginRendering(
                    inheritance_rendering_info,
                )),
                ..Default::default()
            },
        )
        .unwrap();

        secondary_builder
            .bind_pipeline_graphics(pipeline.clone())
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: [game_resolution[0] as f32, game_resolution[1] as f32],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let uniform_buffer = {
            let buffer = vulkan.uniform_buffer_allocator.allocate_sized().unwrap();
            *buffer.write().unwrap() = global_ubo;

            buffer
        };

        let layout = &pipeline.layout().set_layouts()[0];
        let descriptor_set = DescriptorSet::new_variable(
            vulkan.descriptor_set_allocator.clone(),
            layout.clone(),
            resources.textures.len() as u32,
            [
                WriteDescriptorSet::buffer(0, uniform_buffer),
                WriteDescriptorSet::sampler(1, renderer.sampler.clone()),
                WriteDescriptorSet::image_view_array(2, 0, resources.textures.clone()),
            ],
            [],
        )
        .unwrap();

        secondary_builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                descriptor_set,
            )
            .unwrap();

        for object in self.objects_to_render() {
            let push_constant = GamePush {
                model_matrix: object.transform.mat4().to_cols_array_2d(),
                normal_matrix: object.transform.normal_matrix().to_cols_array_2d(),
                color: object.color.to_array(),
                tex_index: object.texture.unwrap_or(-1),
            };

            secondary_builder
                .push_constants(pipeline.layout().clone(), 0, push_constant)
                .unwrap()
                .bind_vertex_buffers(0, object.model.vertex_buffer.clone())
                .unwrap()
                .bind_index_buffer(object.model.index_buffer.clone())
                .unwrap();

            unsafe {
                secondary_builder
                    .draw_indexed(object.model.index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }

        secondary_builder.build().unwrap()
    }

    pub fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(id as usize)
    }

    pub fn get_map(&self) -> &Map {
        &self.map
    }
}
