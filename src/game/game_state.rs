use crate::app_state::{AppState, KeyMap};
use crate::game::Camera;
use crate::game::map::{MapElement, MapSettings};
use crate::game::resources::Resources;
use crate::graphics::object::Object;
use crate::graphics::transform::Transform;
use crate::graphics::{GlobalUbo, Graphics, LightInfo, Renderer, Vulkan};

use glam::{Vec2, Vec3, Vec4};
use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use winit::keyboard::KeyCode;

use crate::game::bomb::Bomb;
use crate::game::input::{Input, InputName, InputState};

use super::map::Map;
use super::player::Player;
use std::error::Error;
use std::sync::Arc;
use std::vec::Vec;

pub struct GameState {
    pub players: Vec<Player>,
    bombs: Vec<Bomb>,
    inputs: Vec<Input>,
    pub resources: Resources,
    pub map: Map,
    pub camera: Camera,
    pub light: LightInfo,
}

impl GameState {
    pub fn default_state(graphics: &Graphics) -> Result<Self, Box<dyn Error>> {
        let resources = Resources::load_resources(
            graphics.vulkan.memory_allocator.clone(),
            graphics.vulkan.command_buffer_allocator.clone(),
            graphics.vulkan.queue.clone(),
        );

        let mut players = Vec::<Player>::new();
        let mut inputs = Vec::<Input>::new();

        //HACK: this is not safe, map can fail creation
        let map = Map::new(MapSettings::default_cheese(), &resources).unwrap();

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

                        inputs.push(Input::default());
                        id += 1;
                    }
                    _ => (),
                }
            }
        }

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
            inputs,
            map,
            resources,
            camera,
            light,
        })
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

        map_objects.chain(players_objects).chain(bomb_objects)
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

    fn update_inputs(&mut self, keys: &KeyMap) {
        //HACK: manually mapping inputs of P1 for testing
        //using F35 as a default
        let mut p1_binds = [KeyCode::F35; 5];
        p1_binds[InputName::Up.value()] = KeyCode::KeyW;
        p1_binds[InputName::Down.value()] = KeyCode::KeyS;
        p1_binds[InputName::Left.value()] = KeyCode::KeyA;
        p1_binds[InputName::Right.value()] = KeyCode::KeyD;
        p1_binds[InputName::Bomb.value()] = KeyCode::Enter;

        self.inputs[0].update_input_player(keys, p1_binds);
    }

    fn mp_game_tick(&mut self, delta: f32, keys: &KeyMap) {
        //update inputs
        self.update_inputs(keys);
        // tick bombs
        for bomb in &mut self.bombs {
            bomb.tick(delta, &mut self.players, &mut self.map, &self.resources);
        }
        self.bombs.retain(|bomb| bomb.despawn == false);
        // for player in players: summon bomb if Pressed
        for (i, player) in self.players.iter_mut().enumerate() {
            if !player.alive {
                continue;
            }
            if self.inputs[i].bomb() == InputState::Pressed {
                match player.create_bomb(&self.resources) {
                    Some(bomb) => self.bombs.push(bomb),
                    None => (),
                }
            }
        }
        for (i, player) in self.players.iter_mut().enumerate() {
            player.player_move(self.inputs[i], delta, &self.map, &self.bombs);
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

    pub fn tick(
        &mut self,
        delta_time: f32,
        keys: &KeyMap,
        window_size: (u32, u32),
    ) -> (Option<AppState>, u8) {
        // let state_func = match self.mode {
        //     Mode::MpGame => Self::mp_game_tick,
        // };
        self.mp_game_tick(delta_time, keys);
        self.update_camera(window_size.0 as f32 / window_size.1 as f32);

        //TODO return new AppState if needed and number of elements to pop from appstate_stack
        (None, 0)
    }

    pub fn render(&self, renderer: &Renderer, vulkan: &Vulkan) -> Arc<SecondaryAutoCommandBuffer> {
        let global_ubo = GlobalUbo {
            projection: self.camera.projection_matrix.to_cols_array_2d(),
            view: self.camera.view_matrix.to_cols_array_2d(),
            inverse_view: self.camera.inverse_view_matrix.to_cols_array_2d(),
            ambient_light_color: self.light.ambient_light_color.into(),
            direction_to_light: self.light.direction_to_light.to_array().into(),
            directional_light_color: self.light.directional_light_color.into(),
        };
        renderer.game_render_system().render_game_objects(
            vulkan,
            renderer.rcx().render_pass.clone(),
            renderer.window_size(),
            self,
            global_ubo,
        )
    }
}
