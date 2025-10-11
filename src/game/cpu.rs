use crate::game::direction::Direction;
use crate::game::map::map::Map;
use crate::game::{self, direction};
use crate::game::{game_state::GameState, player::Player};
use crate::input::input::Input;
use crate::input::input_name::InputName;
use crate::utils::vec2::{ApproxEq, Grid};
use glam::{Vec2, Vec3};
use rand::seq::IndexedRandom;

const CPU_DECISION_TIMER: f32 = 0.1;
const CPU_REACTION_TIME: f32 = 0.9;

//
#[derive(Debug, Clone)]
pub struct CPU {
    pub id: u32,
    last_input: Input,
    path: Vec<Vec2>,
    state: CPUState,
    strategy: CPUStrategy,
    target: Option<Vec2>,
}

/// Possible CPU states.
///
/// Idle: Not moving, a passive state that's mostly used when the CPU is "thinking"
/// Mining: Expending his "playing arena" and looking for powerup by destroying target crate
/// Attacking: Putting a bomb in target
/// Surviving: Trying to survive by moving to target
#[derive(Debug, Clone)]
pub enum CPUState {
    Idle,
    Moving,
    Mining,
    Attacking,
    Surviving,
}

// Different CPU strategy
//
#[derive(Debug, Clone)]
pub enum CPUStrategy {
    Basic,
}

impl CPU {
    // TODO: Check if id is correct via gamestate
    pub fn new(id: u32) -> Self {
        CPU {
            id,
            last_input: Input::default(),
            state: CPUState::Idle,
            target: None,
            path: Vec::new(),
            strategy: CPUStrategy::Basic,
        }
    }

    pub fn get_input(&mut self, state: &GameState) -> Input {
        let map: &Map = state.get_map();
        let player: &Player = state.get_player(self.id).unwrap();
        match self.state {
            CPUState::Moving => {
                if let Some(input) = self.travel(map, player) {
                    input
                } else {
                    self.state = CPUState::Idle;
                    self.do_nothing()
                }
            }
            CPUState::Idle => {
                let position = player.position;
                let neighbours = map.get_neighbours(position.grid());
                self.path = vec![*neighbours.choose(&mut rand::rng()).unwrap(); 1];
                self.state = CPUState::Moving;
                self.last_input
            } /* TODO: This is obviously a placeholder */
            _ => self.do_nothing(),
        }
    }

    fn travel(&mut self, map: &Map, player: &Player) -> Option<Input> {
        let start: &Vec2 = &player.position;
        let goal: &Vec2 = self.path.first()?;
        if start.approx_eq(goal) {
            self.path.remove(0);
            return self.travel(map, player);
        }
        let direction: Direction = Direction::get_direction(start, goal);
        self.set_input(InputName::direction_to_input(direction));
        Some(self.last_input)
    }

    fn do_nothing(&mut self) -> Input {
        self.last_input.release_all();
        self.last_input
    }
    fn set_input(&mut self, input: InputName) {
        self.last_input.release_all_but(input);
        self.last_input.update_input_component(true, input);
    }
}
