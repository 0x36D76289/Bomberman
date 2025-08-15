use core::f32;

use glam::{usize, Vec2, Vec3};

use super::collision::Collision;
use crate::{
    game::{
        direction::Direction,
        map::{Map, MapElement},
        player::Player,
        resources::{ResourceName, Resources},
    },
    graphics::{
        object::{Object, TextureIndex},
        transform::Transform,
    },
};

#[allow(unused)]
pub enum BombState {
    // TODO: Define the states of a bomb
    Planted,
    Sliding(Direction),
    Exploding,
}

#[derive(Default, Debug)]
pub struct Explosion {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

pub struct Bomb {
    pub position: Vec2,
    pub timer: f32,
    pub power: u8,
    pub owner_id: u32,
    pub state: BombState,
    pub collision_enabled: bool,
    pub explosion: Explosion,
    pub despawn: bool,
    // TODO: I'm thinking that when the explosion starts it'll place all the explosion bits in
    // there ?
    pub objects: Vec<Object>,
}

const BOMB_TIMER_DEFAULT: f32 = 3.0;
const BOMB_POWER_DEFAULT: u8 = 2;
const BOMB_EXPLOSION_TIME: f32 = 2.0;
const BOMB_EXPLOSION_RADIUS: f32 = 0.4;
const BOMB_RADIUS: f32 = 0.5;

impl Bomb {
    pub fn new(owner: u32, x: usize, y: usize, resources: &Resources) -> Self {
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
            objects: vec![Self::create_object(x, y, resources)],
        }
    }

    fn create_object(x: usize, y: usize, resources: &Resources) -> Object {
        Object {
            model: resources.models[ResourceName::Bomb as usize].clone(),
            texture: Some(ResourceName::Bomb as TextureIndex),
            color: Vec3::ONE,
            transform: Transform {
                translation: Vec3::new(x as f32 + 0.5, 0.0, y as f32 + 0.5),
                scale: Vec3::splat(0.5),
                rotation: Vec3::ZERO,
            },
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

    fn find_wall(&self, map: &mut Map, dirvec: Vec2) -> u8 {
        for i in 1..self.power + 1 {
            let pos = self.position + dirvec * i as f32;
            let elem = map.get_elem_pos(pos);
            match elem {
                MapElement::Empty => continue,
                MapElement::Breakable(_) => {
                    let _ = map.set_elem_pos(pos, MapElement::Empty);
                }
                MapElement::Unbreakable(_) => (),
            }
            return i - 1;
        }
        self.power
    }

    fn set_explosion_objects(&mut self, resources: &Resources) {
        self.objects.clear();
        for y in -(self.explosion.up as i16)..=(self.explosion.down as i16) {
            self.objects.push(Self::create_object(
                self.position.x as usize,
                (self.position.y as i16 + y) as usize,
                resources,
            ));
        }
        for x in -(self.explosion.left as i16)..=(self.explosion.right as i16) {
            self.objects.push(Self::create_object(
                (self.position.x as i16 + x) as usize,
                self.position.y as usize,
                resources,
            ));
        }
    }

    fn explode(&mut self, map: &mut Map, resources: &Resources) {
        self.state = BombState::Exploding;
        self.center();
        self.explosion.up = self.find_wall(map, Vec2 { x: 0.0, y: -1.0 });
        self.explosion.down = self.find_wall(map, Vec2 { x: 0.0, y: 1.0 });
        self.explosion.left = self.find_wall(map, Vec2 { x: -1.0, y: 0.0 });
        self.explosion.right = self.find_wall(map, Vec2 { x: 1.0, y: 0.0 });
        self.set_explosion_objects(resources);
    }

    fn live_bomb(&mut self, delta: f32, map: &mut Map, resources: &Resources) {
        if (self.timer == 0.0) || (delta >= self.timer) {
            self.timer = 0.0;
            self.explode(map, resources);
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

        // kill players
        for player in players.iter_mut().filter(|p| p.alive) {
            let mut kill = false;

            let (px, py) = player.position.into();
            let (bx, by) = self.position.into();

            if (px - bx).abs() < (BOMB_EXPLOSION_RADIUS + player.get_size()) {
                if ((py + player.get_size())
                    > (by - self.explosion.up as f32 - BOMB_EXPLOSION_RADIUS))
                    && ((py - player.get_size())
                        < (by + self.explosion.down as f32 + BOMB_EXPLOSION_RADIUS))
                {
                    kill = true;
                }
            }
            if (py - by).abs() < (BOMB_EXPLOSION_RADIUS + player.get_size()) {
                if ((px + player.get_size())
                    > (bx - self.explosion.left as f32 - BOMB_EXPLOSION_RADIUS))
                    && ((px - player.get_size())
                        < (bx + self.explosion.right as f32 + BOMB_EXPLOSION_RADIUS))
                {
                    kill = true;
                }
            }

            if kill {
                player.kill();
            }
        }
    }

    pub fn tick(
        &mut self,
        delta: f32,
        players: &mut Vec<Player>,
        map: &mut Map,
        resources: &Resources,
    ) {
        match self.state {
            BombState::Planted => self.live_bomb(delta, map, resources),
            BombState::Sliding(_) => {
                // self.slide(direction);
                self.live_bomb(delta, map, resources);
            }
            BombState::Exploding => self.exploding_bomb(delta, players),
        }
        self.enable_collision(players);
    }
}

impl Collision for Bomb {
    fn get_pos(&self) -> Vec2 {
        self.position
    }
    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos
    }
    fn get_size(&self) -> f32 {
        match self.state {
            BombState::Exploding => 0.0,
            _ => BOMB_RADIUS,
        }
    }
}
