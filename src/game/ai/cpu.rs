use std::cmp::Ordering;
use std::path;
use std::sync::{Arc, Mutex};

use crate::game::ai::ai::AI;
use crate::game::ai::zone::Zone;
use crate::game::bomb::Bomb;
use crate::game::direction::Direction;
use crate::game::map::map::Map;
use crate::game::powerup::PowerUp;
use crate::game::{self, direction};
use crate::game::{game_state::GameState, player::Player};
use crate::input::input::Input;
use crate::input::input_name::InputName;
use crate::utils::vec2::{ApproxEq, Grid};
use glam::{Vec2, Vec3};
use log::logger;
use rand::seq::IndexedRandom;
use rand::{Rng, random_range};

const CPU_DECISION_TIMER: f32 = 0.1;
const CPU_REACTION_TIME: f32 = 0.9;
const CPU_DISTANCE_PWUP: i32 = 5;
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
    action_timer: f32,
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
    Thinking,
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
            action_timer: 0.0,
        }
    }

    pub fn update_zone(&mut self, pos: Vec2, players: &[Player], map: &Map) -> Vec<usize> {
        // log::debug!("I'm cpu {} and i'm updating my zone!", self.id);
        if let Ok(mut zone) = self.zone.lock() {
            zone.fill_zone(pos.grid(), players, map)
        } else {
            Vec::new()
        }
    }

    pub fn get_input(
        &mut self,
        bombs: &[Bomb],
        powerups: &[PowerUp],
        players: &[Player],
        map: &Map,
        delta: f32,
    ) -> Input {
        let player = &players[self.id];
        let mut dangerous_cells: Vec<Vec2> = Vec::new();
        let mut accessible_cells: Vec<Vec2> = Vec::new();
        if let Ok(mut zone) = self.zone.lock() {
            dangerous_cells = zone.check_dangerous_cells(bombs, map);
            accessible_cells = zone.get_accessible_cells(player.position, map, bombs);
            if (!self.path.is_empty()) {
                // log::debug!(
                //     "! == target: {1:?}, papath{0:?}, dangerous : {dangerous_cells:?}; accessible: {accessible_cells:?}",
                //     self.path,
                //     self.target
                // );
            }
        }
        // log::debug!("!{dangerous_cells:?} ! CAREFUL");
        if dangerous_cells.contains(&player.position.grid()) {
            self.path.clear();
            self.target = None;
            // log::debug!("! I'm surviving now");
            self.state = CPUState::Surviving;
        }
        if !self.path.is_empty()
            && self
                .path
                .iter()
                .any(|road| !accessible_cells.contains(road))
        {
            self.path.clear();
            self.target = None;
            log::debug!("! Got cut off, need to rethink");
            self.state = CPUState::Thinking;
        }
        let mut randomness: u8 = rand::rng().random();
        randomness = (randomness + 100) % 100;
        match self.state {
            CPUState::Moving => {
                if self.path.is_empty() {
                    self.target = None;
                    self.state = CPUState::Thinking;
                    self.do_nothing();
                }
                if dangerous_cells.iter().any(|cell| self.path.contains(cell)) {
                    self.path.clear();

                    log::debug!("!my path is now dangerous. I'm surviving now");
                    self.state = CPUState::Surviving;
                    self.target = None;
                    self.do_nothing()
                } else if let Some(input) = self.travel(map, player) {
                    input
                } else {
                    self.action_timer = rand::rng().random_range(0.1..=0.8);
                    self.state = CPUState::Thinking;
                    self.do_nothing()
                }
            }
            CPUState::Idle => {
                if let Ok(mut zone) = self.zone.lock() {
                    if let Some(powerup_pos) = zone.closest_powerup(player.position, powerups) {
                        log::debug!("I'm going to the powerup !");
                        self.target = Some(powerup_pos);
                        self.state = CPUState::Moving;
                    } else if let Some(player_pos) = zone.closest_player(player.position, players)
                        && randomness < 3
                    {
                        log::debug!("I'm going to the player !");
                        self.target = Some(player_pos);
                        self.state = CPUState::Moving;
                    } else if player.bombs_remaining > 0 && randomness < 49 {
                        log::debug!("let's go mining");
                        if let Some(mining_spot) =
                            Zone::find_mining_spot(player.position, &accessible_cells, map)
                        {
                            self.state = CPUState::Mining;
                            self.target = Some(mining_spot);
                        }
                    } else if !accessible_cells.is_empty() {
                        if let Some(random_target) = accessible_cells.choose(&mut rand::rng()) {
                            if random_target.grid() != player.position.grid() {
                                log::debug!("I'm going to {random_target:?}!");
                                self.target = Some(*random_target);
                                self.state = CPUState::Moving;
                            }
                        }
                    }
                }
                if let Some(target) = self.target {
                    if let Some(path) = AI::find_path(player.position.grid(), target, map) {
                        self.path = path;
                        // if let Some(last_cell ) = self.path.last_mut(){
                        //     last_cell.x += rand::rng().random_range(-0.1..0.12);
                        //     last_cell.y += rand::rng().random_range(-0.1..0.12);
                        // }
                        // self.state = CPUState::Moving;
                    } else {
                        self.target = None;
                    }
                }
                return self.last_input;
            }
            CPUState::Surviving => {
                if self.path.is_empty() {
                    let safe_spot = {
                        let safe_cells = accessible_cells
                            .iter()
                            .filter(|cell| !dangerous_cells.contains(&cell.grid()));
                        // log::debug!("Safe cells = {safe_cells:?}");
                        safe_cells.min_by(|a, b| {
                            let dist_a = a.distance_squared(player.position);
                            let dist_b = b.distance_squared(player.position);
                            dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
                        })
                    };
                    // log::debug!("=== Trying to survive == ");
                    if let Some(target_pos) = safe_spot {
                        if let Some(path) = AI::find_path(player.position.grid(), *target_pos, map)
                        {
                            self.path = path;
                        }
                    }
                    // log::debug!("=== Safe spot {safe_spot:?} / path {0:?}== ", self.path);
                }
                if let Some(input) = self.travel(map, player) {
                    input
                } else {
                    self.action_timer = rand::rng().random_range(0.5..=0.7);
                    self.state = CPUState::Thinking;
                    self.do_nothing()
                }
            }
            CPUState::Mining => {
                if dangerous_cells.iter().any(|cell| self.path.contains(cell)) {
                    self.path.clear();

                    log::debug!("!my path is now dangerous. I'm surviving now");
                    self.state = CPUState::Surviving;
                    self.target = None;
                    return self.do_nothing();
                }
                log::debug!("== WE mine target {0:?}, {1:?}", self.target, self.path);
                if let Some(input) = self.travel(map, player) {
                    input
                } else {
                    self.target = None;
                    self.state = CPUState::Thinking;
                    self.set_input(InputName::Bomb);
                    self.last_input
                }
            }
            CPUState::Thinking => {
                self.action_timer -= delta;
                if self.action_timer <= 0.0 {
                    self.state = CPUState::Idle;
                }
                self.do_nothing()
            }
            _ => self.do_nothing(),
        }
    }

    fn travel(&mut self, map: &Map, player: &Player) -> Option<Input> {
        let start: Vec2 = player.position;
        let goal: &Vec2 = self.path.first()?;
        // log::debug!("I'm cpbu {} and traveling {} => {}!", self.id, start, goal);

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
