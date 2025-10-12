use std::sync::{Arc, Mutex};

use crate::game::ai::ai::AI;
use crate::game::ai::zone::Zone;
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
    pub id: usize,
    pub zone: Arc<Mutex<Zone>>,
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
    pub fn new(id: usize) -> Self {
        CPU {
            id,
            zone: Arc::new(Zone::default().into()),
            last_input: Input::default(),
            state: CPUState::Idle,
            target: None,
            path: Vec::new(),
            strategy: CPUStrategy::Basic,
        }
    }

    pub fn update_zone(&mut self, id: usize, players: &[Player], map: &Map) -> Vec<usize> {
        if let Ok(mut zone) = self.zone.lock() {
            zone.fill_zone(players[id].position.grid(), players, map)
        } else {
            Vec::new()
        }
    }

    pub fn get_input(&mut self, players: &[Player], map: &Map) -> Input {
        let player = &players[self.id];

        match self.state {
            CPUState::Moving => {
                if self.path.is_empty() {
                    self.state = CPUState::Idle;
                    self.target = None;
                    self.do_nothing()
                } else if let Some(input) = self.travel(map, player) {
                    input
                } else {
                    self.state = CPUState::Idle;
                    self.do_nothing()
                }
            }
            CPUState::Idle => {
                if let Ok(zone) = self.zone.lock() {
                    if !zone.cells.is_empty() {
                        if let Some(random_target) = zone.cells.choose(&mut rand::rng()) {
                            if random_target.grid() != player.position.grid() {
                                self.target = Some(*random_target);
                            }
                        }
                    }
                }

                if let Some(target) = self.target {
                    if let Some(path) = AI::find_path(player.position.grid(), target, map) {
                        self.path = path;
                        self.state = CPUState::Moving;
                    } else {
                        self.target = None;
                    }
                }

                self.last_input.clone()
            }
            _ => self.do_nothing(),
        }
    }

    fn travel(&mut self, map: &Map, player: &Player) -> Option<Input> {
        let start: Vec2 = player.position.grid();
        let goal: &Vec2 = self.path.first()?;

        if start.approx_eq(goal) {
            self.path.remove(0);
            return self.travel(map, player);
        }

        let direction: Direction = Direction::get_direction(&player.position, goal);
        self.set_input(InputName::direction_to_input(direction));
        Some(self.last_input.clone())
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
