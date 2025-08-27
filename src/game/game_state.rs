use crate::app_state::{AppState, KeyMap};
use crate::game::bomb::{Bomb, BombState};
use crate::game::game_settings::GameSettings;
use crate::game::map::map::Map;
use crate::game::map::map_element::MapElement;
use crate::game::map::map_settings::MapSettings;
use crate::game::player::Player;
use crate::game::powerup::PowerUp;
use crate::game::resources::Resources;
use crate::game::Camera;
use crate::graphics::object::Object;
use crate::graphics::transform::Transform;
use crate::graphics::{GamePush, GlobalUbo, LightInfo, Renderer, Vulkan};
use crate::input::input::Input;
use crate::input::input_state::InputState;
use crate::input::input_vec::GetOrDefault;
use crate::ui::UiState;
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

#[derive(Debug, Clone)]
pub struct GameState {
    players: Vec<Player>,
    game_inputs: Vec<Input>,
    nb_humans: u32,
    bombs: Vec<Bomb>,
    power_ups: Vec<PowerUp>,
    map: Map,
    camera: Transform,
    light: LightInfo,
}

impl GameState {
    fn create_players(map: &Map, resources: &Resources, nb_humans: &u32) -> Vec<Player> {
        let mut players = Vec::<Player>::new();
        let mut id: u32 = 0;
        for spawn in map.spawns.clone() {
            players.push(Player::new(
                id,
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

    //TODO: add get_input_player -> returns Released if p doesn't exist
    pub fn default_state(
        resources: &Resources,
        settings: GameSettings,
    ) -> Result<Self, Box<dyn Error>> {
        //HACK: this is not safe, map can fail creation
        //LOIC: true
        let map = Map::new(settings.map_settings, &resources).unwrap();
        let nb_humans = settings.nb_humans;
        let players = Self::create_players(&map, &resources, &nb_humans);
        let game_inputs = vec![Input::default(); players.len()];

        let camera = Transform {
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
            game_inputs,
            nb_humans,
            bombs: Vec::<Bomb>::new(),
            power_ups: Vec::<PowerUp>::new(),
            map,
            camera,
            light,
        })
    }

    fn recreate(&self, resources: &Resources) -> Self {
        let map = Map::new(MapSettings::default(), resources).unwrap();
        let players = Self::create_players(&map, &resources, &self.nb_humans);
        Self {
            players,
            game_inputs: self.game_inputs.clone(),
            nb_humans: self.nb_humans,
            bombs: Vec::<Bomb>::new(),
            power_ups: Vec::<PowerUp>::new(),
            map,
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
                MapElement::Breakable(obj) => Some(obj),
                MapElement::Unbreakable(obj) => Some(obj),
            })
            .chain(std::iter::once(&self.map.floor));

        let players_objects = self.players.iter().flat_map(|p| &p.object);
        let bomb_objects = self.bombs.iter().flat_map(|b| &b.objects);
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

    fn mp_game_tick(&mut self, delta: f32, resources: &Resources) {
        // tick bombs
        for bomb in &mut self.bombs {
            bomb.tick(
                delta,
                &mut self.players,
                &mut self.map,
                &mut self.power_ups,
                resources,
            );
        }
        for i in 0..self.bombs.len() {
            if self.bombs[i].state != BombState::Exploding {
                continue;
            }
            self.bombs[i].clone().chain_react(&mut self.bombs);
        }
        self.bombs.retain(|bomb| !bomb.despawn);
        //tick powerups
        for powerup in &mut self.power_ups {
            powerup.tick(&mut self.players);
        }
        self.power_ups.retain(|powerup| !powerup.despawn);
        // for player in players: summon bomb if Pressed
        for (i, player) in self.players.iter_mut().enumerate() {
            if !player.alive {
                continue;
            }
            if self.game_inputs.get_or_default(i).bomb() == InputState::Pressed
                && let Some(bomb) = player.create_bomb(&resources, &self.bombs)
            {
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
        // uncomment this and comment the previous line to control the camera
        // self.camera.keyboard_move(&self.game_inputs[0], delta);
    }

    fn restart_inside(&mut self, keys: &KeyMap, resources: &Resources) {
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
                resources,
            )
            .unwrap();
            self.players = Self::create_players(&self.map, resources, &self.nb_humans);
            self.bombs = Vec::new();
        }
        *was_pressed = is_pressed;
    }

    pub fn tick(
        &mut self,
        delta_time: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
    ) -> (Option<AppState>, u8) {
        //Pause
        if keys
            .get(&PhysicalKey::Code(KeyCode::Escape))
            .unwrap_or(&winit::event::ElementState::Released)
            .is_pressed()
        {
            return (Some(AppState::Ui(UiState::pause())), 0);
        }

        #[cfg(debug_assertions)]
        self.restart_inside(keys, resources);

        // let state_func = match self.mode {
        //     Mode::MpGame => Self::mp_game_tick,
        // };
        self.inputs_to_game_inputs(inputs);
        self.mp_game_tick(delta_time, resources);

        #[cfg(debug_assertions)]
        if keys
            .get(&PhysicalKey::Code(KeyCode::KeyT))
            .unwrap_or(&winit::event::ElementState::Released)
            .is_pressed()
        {
            return (Some(AppState::Game(self.recreate(resources))), 1);
        }

        //TODO: return new AppState if needed and number of elements to pop from appstate_stack
        (None, 0)
    }

    // Put the inputs read into game inputs
    fn inputs_to_game_inputs(&mut self, inputs: &Vec<Input>) {
        for (i, input) in inputs.iter().enumerate() {
            self.game_inputs[i] = input.clone();
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
                "Called render on a GameState object but the world_pipeline is not initialized in the renderer"
            ),
        };

        let window_size = renderer.window_size();

        let mut camera = Camera::new();
        camera.set_view_xyz(self.camera.translation, self.camera.rotation);
        camera.set_perspective_projection(
            0.6,
            window_size[0] as f32 / window_size[1] as f32,
            0.1,
            100.0,
        );

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
