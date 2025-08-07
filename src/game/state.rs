use glam::Vec3;
use vulkano::image::view::ImageView;

use crate::game::{Camera, Entity};
use crate::input::{InputState as SamyInputState, KeyboardMovementController};

use glam::Vec2;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::game::bomb::Bomb;
use crate::game::direction::Direction;
use crate::game::input::{Input, InputState};
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
    pub map: Map,
    pub entities: Vec<Entity>,
    pub textures: Vec<Arc<ImageView>>,
    pub camera: Camera,
    pub entity_controller: KeyboardMovementController,
    pub controlled_object_id: usize,
    pub fps: FpsManager,
    mode: Mode,
}

impl State {
    pub fn default_state(
        entities: Vec<Entity>,
        textures: Vec<Arc<ImageView>>,
    ) -> Result<Self, Box<dyn Error>> {
        let input_state = SamyInputState::default();

        // let players = Vec::new();

        // let map = Map::new(16, 16);

        let mut camera = Camera::new();
        camera.set_view_target(Vec3::new(1.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0));

        let entity_controller = KeyboardMovementController {
            move_speed: 3.0,
            look_speed: 1.5,
        };

        let mut players = Vec::<Player>::new();
        players.push(Player::new(0, Vec2 { x: 1.5, y: 1.5 }, Direction::Down));
        let mut inputs = Vec::<Input>::new();
        inputs.push(Input::default());

        Ok(Self {
            keys: HashMap::<PhysicalKey, ElementState>::new(),
            input_state: input_state,
            players: players,
            bombs: Vec::<Bomb>::new(),
            inputs: inputs,
            map: Map::new(16, 16),
            entities: entities,
            textures: textures,
            camera: camera,
            entity_controller: entity_controller,
            controlled_object_id: 1,
            fps: FpsManager::default(),
            mode: Mode::MpGame,
        })
    }

    pub fn debug(&self) {
        for entity in self.entities.iter() {
            println!("{entity:?}");
        }
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
        match self.keys.get(&PhysicalKey::Code(KeyCode::KeyW)) {
            Some(state) => {
                if state.is_pressed() {
                    if self.inputs[0].up == InputState::Released {
                        self.inputs[0].up = InputState::Pressed;
                    } else {
                        self.inputs[0].up = InputState::Held;
                    }
                } else {
                    self.inputs[0].up = InputState::Released;
                }
            }
            None => (),
        }
        match self.keys.get(&PhysicalKey::Code(KeyCode::KeyS)) {
            Some(state) => {
                if state.is_pressed() {
                    if self.inputs[0].down == InputState::Released {
                        self.inputs[0].down = InputState::Pressed;
                    } else {
                        self.inputs[0].down = InputState::Held;
                    }
                } else {
                    self.inputs[0].down = InputState::Released;
                }
            }
            None => (),
        }
        match self.keys.get(&PhysicalKey::Code(KeyCode::KeyA)) {
            Some(state) => {
                if state.is_pressed() {
                    if self.inputs[0].left == InputState::Released {
                        self.inputs[0].left = InputState::Pressed;
                    } else {
                        self.inputs[0].left = InputState::Held;
                    }
                } else {
                    self.inputs[0].left = InputState::Released;
                }
            }
            None => (),
        }
        match self.keys.get(&PhysicalKey::Code(KeyCode::KeyD)) {
            Some(state) => {
                if state.is_pressed() {
                    if self.inputs[0].right == InputState::Released {
                        self.inputs[0].right = InputState::Pressed;
                    } else {
                        self.inputs[0].right = InputState::Held;
                    }
                } else {
                    self.inputs[0].right = InputState::Released;
                }
            }
            None => (),
        }
        match self.keys.get(&PhysicalKey::Code(KeyCode::Enter)) {
            Some(state) => {
                if state.is_pressed() {
                    if self.inputs[0].bomb == InputState::Released {
                        self.inputs[0].bomb = InputState::Pressed;
                    } else {
                        self.inputs[0].bomb = InputState::Held;
                    }
                } else {
                    self.inputs[0].bomb = InputState::Released;
                }
            }
            None => (),
        }
    }

    fn mp_game_tick(&mut self, delta: f32) {
        //update inputs
        self.update_inputs();
        // tick bombs
        for bomb in &mut self.bombs {
            bomb.tick(delta, &mut self.players, &mut self.map);
        }
        self.bombs.retain(|bomb| bomb.despawn == false);
        // for player in players: summon bomb if Pressed
        for i in 0..self.players.len() {
            if self.inputs[i].bomb == InputState::Pressed {
                match self.players[i].create_bomb() {
                    Some(bomb) => self.bombs.push(bomb),
                    None => (),
                }
            }
        }
        for i in 0..self.players.len() {
            self.players[i].player_move(self.inputs[i], delta, &self.map, &self.bombs);
        }
    }

    pub fn tick(&mut self) {
        let state_func = match self.mode {
            Mode::MpGame => Self::mp_game_tick,
        };
        state_func(self, self.fps.get_delta());
    }
}

// impl Default for State {
//     fn default() -> Self {
//         Self::new()
//     }
// }
