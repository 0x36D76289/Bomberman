use crate::app_state::{AppState, KeyMap};
use crate::game::bomb::{Bomb, BombState};
use crate::game::map::map::Map;
use crate::game::map::map_element::MapElement;
use crate::game::map::map_settings::MapSettings;
use crate::game::player::Player;
use crate::game::powerup::PowerUp;
use crate::game::resources::Resources;
use crate::game::Camera;
use crate::graphics::object::Object;
use crate::graphics::transform::Transform;
use crate::graphics::{GlobalUbo, Graphics, LightInfo, Push, Renderer, Vulkan};
use crate::input::input::{GetOrDefault, Input};
use crate::input::input_state::InputState;
use glam::{bool, Vec2, Vec3, Vec4};
use rand::random_range;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use std::vec::Vec;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferInheritanceInfo, CommandBufferInheritanceRenderPassType,
    CommandBufferInheritanceRenderingInfo, CommandBufferUsage, SecondaryAutoCommandBuffer,
};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{Pipeline, PipelineBindPoint};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct GameState {
    players: Vec<Player>,
    bombs: Vec<Bomb>,
    power_ups: Vec<PowerUp>,
    pub resources: Resources,
    map: Map,
    camera: Camera,
    light: LightInfo,
}

impl GameState {
    fn create_players(map: &Map, resources: &Resources) -> Vec<Player> {
        let mut players = Vec::<Player>::new();
        let mut id: u32 = 0;

        for y in 0..map.height {
            for x in 0..map.width {
                match map.get_elem(x, y) {
                    MapElement::SpawnPoint(dir) => {
                        players.push(Player::new(
                            id,
                            Vec2 {
                                x: x as f32 + 0.5,
                                y: y as f32 + 0.5,
                            },
                            dir.clone(),
                            &resources,
                        ));

                        id += 1;
                    }
                    _ => (),
                }
            }
        }
        players
    }

    //TODO: add get_input_player -> returns Released if p doesn't exist
    pub fn default_state(graphics: &Graphics) -> Result<Self, Box<dyn Error>> {
        let resources = Resources::load_resources(
            graphics.vulkan.memory_allocator.clone(),
            graphics.vulkan.command_buffer_allocator.clone(),
            graphics.vulkan.queue.clone(),
        );

        //HACK: this is not safe, map can fail creation
        let map = Map::new(MapSettings::default(), &resources).unwrap();
        let players = Self::create_players(&map, &resources);

        let mut camera = Camera::new();
        camera.transform = Transform {
            translation: Vec3::new(map.width as f32 / 2.0, -27.5, -1.25),
            scale: Vec3::ONE,
            rotation: Vec3::new(-1.25, 0.0, 0.0),
        };

        let light = LightInfo {
            ambient_light_color: Vec4::ONE.with_w(0.8),
            direction_to_light: Vec3::new(0.0, -3.0, 1.0).normalize(),
            directional_light_color: Vec4::ONE.with_w(0.6),
        };

        Ok(Self {
            players,
            bombs: Vec::<Bomb>::new(),
            power_ups: Vec::<PowerUp>::new(),
            map,
            resources,
            camera,
            light,
        })
    }

    fn recreate(&self) -> Self {
        let map = Map::new(MapSettings::default(), &self.resources).unwrap();
        let players = Self::create_players(&map, &self.resources);
        Self {
            players,
            bombs: Vec::<Bomb>::new(),
            power_ups: Vec::<PowerUp>::new(),
            map,
            resources: self.resources.clone(),
            camera: self.camera,
            light: self.light,
        }
    }

    pub fn objects_to_render(&self) -> impl Iterator<Item = &Object> {
        let map_objects = self
            .map
            .iter()
            .filter_map(|el| match el {
                MapElement::Empty => None,
                MapElement::SpawnPoint(_) => None,
                MapElement::Breakable(obj) => Some(obj),
                MapElement::Unbreakable(obj) => Some(obj),
            })
            .chain(std::iter::once(&self.map.floor));

        let players_objects = self.players.iter().map(|p| &p.object).flatten();
        let bomb_objects = self.bombs.iter().map(|b| &b.objects).flatten();
        let power_up_objects = self.power_ups.iter().map(|p| &p.object);

        map_objects
            .chain(players_objects)
            .chain(bomb_objects)
            .chain(power_up_objects)
    }

    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn print(&self) {
        let mut display = self.map.to_str();
        for player in &self.players {
            println!("player pos: {} {}", player.position.x, player.position.y);
            let y: usize = player.position.y as usize;
            let x: usize = player.position.x as usize;
            println!("player pos: {} {}", x, y);
            let pos: usize = y * (self.map.width + 1) + x;
            display.replace_range(pos..pos + 1, "+");
        }
        for bomb in &self.bombs {
            let y: usize = bomb.position.y as usize;
            let x: usize = bomb.position.x as usize;
            let pos: usize = y * (self.map.width + 1) + x;
            display.replace_range(pos..pos + 1, "O");
        }

        print!("{}", display);
    }

    fn mp_game_tick(&mut self, delta: f32, inputs: &Vec<Input>) {
        // tick bombs
        for bomb in &mut self.bombs {
            bomb.tick(
                delta,
                &mut self.players,
                &mut self.map,
                &mut self.power_ups,
                &self.resources,
            );
        }
        for i in 0..self.bombs.len() {
            if self.bombs[i].state != BombState::Exploding {
                continue;
            }
            self.bombs[i].clone().chain_react(&mut self.bombs);
        }
        self.bombs.retain(|bomb| bomb.despawn == false);
        //tick powerups
        for powerup in &mut self.power_ups {
            powerup.tick(&mut self.players);
        }
        self.power_ups.retain(|powerup| powerup.despawn == false);
        // for player in players: summon bomb if Pressed
        for (i, player) in self.players.iter_mut().enumerate() {
            if !player.alive {
                continue;
            }
            if inputs.get_or_default(i).bomb() == InputState::Pressed {
                match player.create_bomb(&self.resources) {
                    Some(bomb) => self.bombs.push(bomb),
                    None => (),
                }
            }
        }
        // player movement
        for (i, player) in self.players.iter_mut().enumerate() {
            player.player_move(inputs.get_or_default(i), delta, &self.map, &self.bombs);
        }
        // uncomment this and comment the previous line to control the camera
        // self.camera.keyboard_move(&self.inputs[0], delta);
    }

    fn update_camera(&mut self, aspect_ratio: f32) {
        self.camera.set_view_xyz(
            self.camera.transform.translation,
            self.camera.transform.rotation,
        );
        self.camera
            .set_perspective_projection(0.6, aspect_ratio, 0.1, 100.0);
    }

    fn restart_inside(&mut self, keys: &KeyMap) {
        static WAS_PRESSED: Mutex<bool> = Mutex::new(false);

        let mut was_pressed = WAS_PRESSED.lock().unwrap();
        let is_pressed = keys
            .get(&PhysicalKey::Code(KeyCode::KeyR))
            .unwrap_or(&winit::event::ElementState::Released)
            .is_pressed();

        if is_pressed && !*was_pressed {
            //HACK: this is the only part that would be kept if this wasn't a silly bind
            self.map = Map::new(
                MapSettings {
                    spawns: random_range(2..=8),
                    ..MapSettings::default_cheese()
                },
                &self.resources,
            )
            .unwrap();
            self.players = Self::create_players(&self.map, &self.resources);
            self.bombs = Vec::new();
        }
        *was_pressed = is_pressed;
    }

    pub fn tick(
        &mut self,
        delta_time: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        window_size: (u32, u32),
    ) -> (Option<AppState>, u8) {
        #[cfg(debug_assertions)]
        self.restart_inside(keys);

        // let state_func = match self.mode {
        //     Mode::MpGame => Self::mp_game_tick,
        // };
        self.mp_game_tick(delta_time, inputs);
        self.update_camera(window_size.0 as f32 / window_size.1 as f32);

        #[cfg(debug_assertions)]
        if keys
            .get(&PhysicalKey::Code(KeyCode::KeyT))
            .unwrap_or(&winit::event::ElementState::Released)
            .is_pressed()
        {
            return (Some(AppState::Game(self.recreate())), 1);
        }

        //TODO: return new AppState if needed and number of elements to pop from appstate_stack
        (None, 0)
    }

    pub fn render(&self, vulkan: &Vulkan, renderer: &Renderer) -> Arc<SecondaryAutoCommandBuffer> {
        let pipeline = match renderer.world_pipeline.as_ref() {
            Some(pipeline) => pipeline.clone(),
            None => panic!(
                "Called render on a GameState object but the world_pipeline is not initialized in the renderer"
            ),
        };

        let global_ubo = GlobalUbo {
            projection: self.camera.projection_matrix.to_cols_array_2d(),
            view: self.camera.view_matrix.to_cols_array_2d(),
            inverse_view: self.camera.inverse_view_matrix.to_cols_array_2d(),
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
            CommandBufferUsage::SimultaneousUse,
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
                    extent: renderer.rcx().window.inner_size().into(),
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
            self.resources.textures.len() as u32,
            [
                WriteDescriptorSet::buffer(0, uniform_buffer),
                WriteDescriptorSet::sampler(1, renderer.sampler.clone()),
                WriteDescriptorSet::image_view_array(2, 0, self.resources.textures.clone()),
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
            let push_constant = Push {
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
