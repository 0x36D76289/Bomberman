use glam::{usize, Vec2};

use crate::game::player::Player;

pub enum BombState {
    // TODO: Define the states of a bomb
    Planted,
    // Sliding(Direction),
    Exploding,
}

#[derive(Default)]
struct Explosion {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

pub struct Bomb {
    pub position: Vec2,
    pub timer: f32,
    pub power: u32,
    pub owner_id: u32,
    pub state: BombState,
    pub collision_enabled: bool,
    pub explosion: Explosion,
}

const BOMB_TIMER_DEFAULT: f32 = 3.0;
const BOMB_POWER_DEFAULT: u32 = 1;

impl Bomb {
    pub fn new(owner: u32, x: usize, y: usize) -> Self {
        Self {
            position: Vec2 {
                x: x as f32 + 0.5,
                y: y as f32 + 0.5,
            },
            timer: BOMB_TIMER_DEFAULT,
            power: BOMB_POWER_DEFAULT,
            owner_id: owner,
            state: BombState::Planted,
            collision_enabled: false,
            explosion: Explosion::default(),
        }
    }

    fn enable_collision(&mut self, players: &Vec<Player>) {
        if self.collision_enabled {
            return;
        }
        if !players[self.owner_id as usize].is_colliding(self.position, 0.5) {
            self.collision_enabled = true;
        }
    }
    pub fn tick(&mut self, players: &Vec<Player>) {
        self.enable_collision(players);
    }
}
