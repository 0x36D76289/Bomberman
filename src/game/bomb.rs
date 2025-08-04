use core::f32;

use glam::{usize, Vec2};

use super::collision::Collision;
use crate::game::{direction::Direction, player::Player};

pub enum BombState {
    // TODO: Define the states of a bomb
    Planted,
    Sliding(Direction),
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
    pub despawn: bool,
}

const BOMB_TIMER_DEFAULT: f32 = 3.0;
const BOMB_POWER_DEFAULT: u32 = 1;
const BOMB_EXPLOSION_TIME: f32 = 3.0;

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
            despawn: false,
        }
    }

    fn enable_collision(&mut self, players: &Vec<Player>) {
        if self.collision_enabled {
            return;
        }
        if !players[self.owner_id as usize].is_colliding_with(self.position, 0.5) {
            self.collision_enabled = true;
        }
    }

    fn center(&mut self) {
        self.position.x = self.position.x as usize as f32 + 0.5;
        self.position.y = self.position.y as usize as f32 + 0.5;
    }

    fn explode(&mut self) {
        self.state = BombState::Exploding;
        self.center();
        //TODO:
        //FIXME:
        //NOTE:
        //HACK:
        // ADD the thingie where it deletes blocks and creates Explosion object
    }

    fn live_bomb(&mut self, delta: f32) {
        if (self.timer == 0.0) || (delta >= self.timer) {
            self.timer = 0.0;
            self.explode();
            return;
        }
        self.timer -= delta;
    }

    fn exploding_bomb(&mut self, delta: f32, players: &mut Vec<Player>) {
        if self.timer >= BOMB_EXPLOSION_TIME {
            self.despawn = true;
            players[self.owner_id as usize].bombs_remaining += 1;
        }
        self.timer += delta;
        //TODO: kill players
    }

    pub fn tick(&mut self, delta: f32, players: &mut Vec<Player>) {
        match self.state {
            BombState::Planted => self.live_bomb(delta),
            BombState::Sliding(direction) => {
                // self.slide(direction);
                self.live_bomb(delta);
            }
            BombState::Exploding => self.exploding_bomb(delta, players),
        }
        self.enable_collision(players);
    }
}
