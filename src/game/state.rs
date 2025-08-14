use glam::Vec3;
use vulkano::image::view::ImageView;
use vulkano::pipeline::graphics;

use crate::game::Camera;
use crate::game::map::MapElement;
use crate::game::resources::{self, Resources};
use crate::graphics::Graphics;
use crate::graphics::object::Object;
use crate::input::{InputState as SamyInputState, KeyboardMovementController};

use glam::Vec2;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::game::bomb::Bomb;
use crate::game::direction::Direction;
use crate::game::input::{Input, InputName, InputState};
use crate::settings::fps::FpsManager;

use super::map::Map;
use super::player::Player;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    MpGame,
}

pub struct State {
    keys: HashMap<PhysicalKey, ElementState>,
    pub input_state: SamyInputState,
    pub players: Vec<Player>,
    bombs: Vec<Bomb>,
    inputs: Vec<Input>,
    pub resources: Resources,
    pub map: Map,
    pub camera: Camera,
    pub entity_controller: KeyboardMovementController,
    pub controlled_object_id: usize,
    mode: Mode,
}

impl State {
    pub fn default_state(graphics: &Graphics) -> Result<Self, Box<dyn Error>> {
        let input_state = SamyInputState::default();

        // let players = Vec::new();

        // let map = Map::new(16, 16);

        let resources = Resources::load_resources(
            graphics.vulkan.memory_allocator.clone(),
            graphics.vulkan.command_buffer_allocator.clone(),
            graphics.vulkan.queue.clone(),
        );

        let mut camera = Camera::new();

        let entity_controller = KeyboardMovementController {
            move_speed: 3.0,
            look_speed: 1.5,
        };

        let mut players = Vec::<Player>::new();
        players.push(Player::new(
            0,
            Vec2 { x: 1.5, y: 1.5 },
            Direction::Down,
            &resources,
        ));
        let mut inputs = Vec::<Input>::new();
        inputs.push(Input::default());

        let map = Map::new(16, 16, &resources);

        Ok(Self {
            keys: HashMap::<PhysicalKey, ElementState>::new(),
            input_state,
            players,
            bombs: Vec::<Bomb>::new(),
            inputs,
            map,
            resources,
            camera,
            entity_controller: entity_controller,
            controlled_object_id: 1,
            mode: Mode::MpGame,
        })
    }

    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        let map_objects = self
            .map
            .iter()
            .filter_map(|el| match el {
                MapElement::Empty => None,
                MapElement::Breakable(obj) => Some(obj),
                MapElement::Unbreakable(obj) => Some(obj),
            })
            .chain(std::iter::once(&self.map.floor));

        let players_objects = self.players.iter().map(|p| &p.object);
        let bomb_objects = self.bombs.iter().map(|b| &b.objects).flatten();

        map_objects.chain(players_objects).chain(bomb_objects)
    }

    // pub fn new() -> Self {
    //     let mut players = Vec::<Player>::new();
    //     players.push(Player::new(0, Vec2 { x: 1.5, y: 1.5 }, Direction::Down));
    //     let mut inputs = Vec::<Input>::new();
    //     inputs.push(Input::default());
    //     State {
    //         keys: HashMap::<PhysicalKey, ElementState>::new(),
    //         players: players,
    //         bombs: Vec::<Bomb>::new(),
    //         inputs: inputs,
    //         map: Map::new(16, 16),
    //         fps: FpsManager::default(),
    //         mode: Mode::MpGame,
    //     }
    // }
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

    pub fn record_key(&mut self, code: PhysicalKey, state: ElementState) {
        self.keys.insert(code, state);
    }

    fn update_inputs(&mut self) {
        //HACK: manually mapping inputs of P1 for testing
        //using F35 as a default
        let mut p1_binds = [KeyCode::F35; 5];
        p1_binds[InputName::Up.value()] = KeyCode::KeyW;
        p1_binds[InputName::Down.value()] = KeyCode::KeyS;
        p1_binds[InputName::Left.value()] = KeyCode::KeyA;
        p1_binds[InputName::Right.value()] = KeyCode::KeyD;
        p1_binds[InputName::Bomb.value()] = KeyCode::Enter;

        self.inputs[0].update_input_player(&self.keys, p1_binds);
    }

    fn mp_game_tick(&mut self, delta: f32) {
        //update inputs
        self.update_inputs();
        // tick bombs
        for bomb in &mut self.bombs {
            bomb.tick(delta, &mut self.players, &mut self.map, &self.resources);
        }
        self.bombs.retain(|bomb| bomb.despawn == false);
        // for player in players: summon bomb if Pressed
        for i in 0..self.players.len() {
            if self.inputs[i].bomb() == InputState::Pressed {
                match self.players[i].create_bomb(&self.resources) {
                    Some(bomb) => self.bombs.push(bomb),
                    None => (),
                }
            }
        }
        for i in 0..self.players.len() {
            self.players[i].player_move(self.inputs[i], delta, &self.map, &self.bombs);
        }
    }

    pub fn tick(&mut self, delta_time: f32) {
        let state_func = match self.mode {
            Mode::MpGame => Self::mp_game_tick,
        };
        state_func(self, delta_time);
    }
}

// impl Default for State {
//     fn default() -> Self {
//         Self::new()
//     }
// }
