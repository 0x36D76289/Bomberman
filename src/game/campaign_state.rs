use crate::{
    app_state::{AppState, KeyMap},
    audio::{AudioManager, SoundEffect},
    game::{
        bomb::{Bomb, BombState},
        camera::Camera,
        campaign::LEVELS,
        enemy::Enemy,
        map::{map::Map, map_element::MapElement},
        player::Player,
        powerup::PowerUp,
        resources::Resources,
    },
    graphics::{
        GamePush, GlobalUbo, LightInfo, Renderer, Vulkan, object::Object,
        renderer::RENDER_RES_RATIO, transform::Transform,
    },
    input::{input::Input, input_state::InputState, input_vec::GetOrDefault},
    ui::UiState,
};
use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;
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

#[derive(Debug, Clone)]
pub struct CampaignState {
    current_level_index: usize,
    player: Player,
    enemies: Vec<Enemy>,
    bombs: Vec<Bomb>,
    power_ups: Vec<PowerUp>,
    map: Map,
    camera: Transform,
    light: LightInfo,
}

impl CampaignState {
    pub fn new(level_index: usize, resources: &Resources) -> Self {
        let level = &LEVELS[level_index];
        let map = Map::from_layout(level.map_layout, resources);

        let spawn = map
            .spawns
            .first()
            .expect("La carte de campagne doit avoir un spawn pour le joueur.");
        let player = Player::new(
            0,
            Vec2::new(spawn.x as f32 + 0.5, spawn.y as f32 + 0.5),
            spawn.direction,
            resources,
            true,
        );

        let enemies = level
            .enemies
            .iter()
            .map(|spawn| Enemy::new(spawn.start, spawn.end, resources))
            .collect();

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

        Self {
            current_level_index: level_index,
            player,
            enemies,
            bombs: Vec::new(),
            power_ups: Vec::new(),
            map,
            camera,
            light,
        }
    }

    pub fn tick(
        &mut self,
        delta_time: f32,
        inputs: &Vec<Input>,
        _keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        let mut players_vec = vec![self.player.clone()];
        players_vec[0].player_move(
            inputs.get_or_default(0),
            delta_time,
            &self.map,
            &mut self.bombs,
<<<<<<< HEAD
<<<<<<< HEAD
            &mut self.enemies,
=======
>>>>>>> 71022cd (feat: Implement a simple single-player system, with enemies, and pathing)
=======
            &mut self.enemies,
>>>>>>> c19f531 (feat: Add enemy handling to player movement and collision detection)
        );
        if inputs.get_or_default(0).bomb() == InputState::Pressed {
            if let Some(bomb) = players_vec[0].create_bomb(resources, &self.bombs) {
                audio_manager.play_sound_effect(SoundEffect::PutBomb);
                self.bombs.push(bomb);
            }
        }
        self.player = players_vec.remove(0);

        for enemy in &mut self.enemies {
            enemy.update(delta_time, &self.map);
        }

        let mut players_vec_for_bombs = vec![self.player.clone()];
        for bomb in &mut self.bombs {
            bomb.tick(
                delta_time,
                &mut players_vec_for_bombs,
                &mut self.enemies,
                &mut self.map,
                &mut self.power_ups,
                resources,
                audio_manager,
            );
        }
        self.player = players_vec_for_bombs.remove(0);

        for i in 0..self.bombs.len() {
            if self.bombs[i].state != BombState::Exploding {
                continue;
            }
            self.bombs[i].clone().chain_react(&mut self.bombs);
        }
        self.bombs.retain(|bomb| !bomb.despawn);

        let mut players_vec_for_powerups = vec![self.player.clone()];
        for powerup in &mut self.power_ups {
            powerup.tick(&mut players_vec_for_powerups, audio_manager);
        }
        self.player = players_vec_for_powerups.remove(0);
        self.power_ups.retain(|powerup| !powerup.despawn);

        if self.enemies.iter().all(|e| !e.alive) {
            let next_level = self.current_level_index + 1;
            if next_level < LEVELS.len() {
                return (
                    Some(AppState::Campaign(Self::new(next_level, resources))),
                    1,
                );
            } else {
                return (Some(AppState::Ui(UiState::main_menu())), 1);
            }
        }

        (None, 0)
    }

    pub fn objects_to_render(&self) -> impl Iterator<Item = &Object> {
        let map_objects = self
            .map
            .iter()
            .filter_map(|el| match el {
                MapElement::Empty => None,
                MapElement::Breakable(obj) => Some(obj),
                MapElement::Unbreakable(obj) => Some(obj),
            })
            .chain(std::iter::once(&self.map.floor));

        let player_object = self.player.object.iter();
        let bomb_objects = self.bombs.iter().flat_map(|b| &b.objects);
        let power_up_objects = self.power_ups.iter().map(|p| &p.object);
        let enemy_objects = self.enemies.iter().filter_map(|e| e.object.as_ref());

        map_objects
            .chain(player_object)
            .chain(bomb_objects)
            .chain(power_up_objects)
            .chain(enemy_objects)
    }

    pub fn render(
        &self,
        vulkan: &Vulkan,
        renderer: &Renderer,
        resources: &Resources,
    ) -> Arc<SecondaryAutoCommandBuffer> {
        let pipeline = renderer.game_pipeline.as_ref().unwrap().clone();
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
}
