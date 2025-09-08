use crate::{
    game::{
        bomb::{Bomb, BombState},
        collision::Collision,
        map::map::Map,
        resources::{ResourceName, Resources},
        enemy::Enemy,
    },
    graphics::{object::Object, transform::Transform},
    input::input::Input,
};

use super::direction::Direction;
use glam::{Vec2, Vec3};

const PLAYER_RADIUS: f32 = 0.4;
const PLAYER_SPEEDS: [f32; 4] = [2.0, 3.5, 5.0, 6.0];

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub position: Vec2,
    pub direction: Direction,
    pub alive: bool,
    pub power_level: u8,
    pub speed_level: u8,
    pub bombs_remaining: u32,
    pub is_human: bool,
    pub can_kick_bomb: bool,
    pub object: Option<Object>,
}

impl Player {
    pub fn new(
        id: u32,
        position: Vec2,
        direction: Direction,
        resources: &Resources,
        is_human: bool,
    ) -> Self {
        let dir_vec = direction.to_vec2();
        Player {
            id,
            position,
            direction,
            alive: true,
            power_level: 2,
            speed_level: 0,
            bombs_remaining: 1,
            is_human,
            can_kick_bomb: false,
            object: Some(Object {
                model: resources.models[&ResourceName::Player].clone(),
                texture: Some(resources.textures_index[&ResourceName::Player]),
                color: Vec3::ONE,
                transform: Transform {
                    translation: Vec3::new(position.x, 0.0, position.y),
                    scale: Vec3::splat(0.35),
                    rotation: Vec3::new(0.0, dir_vec.x.atan2(dir_vec.y), 0.0),
                },
            }),
        }
    }

    fn handle_collisions(&mut self, map: &Map, direction: Direction, bombs: &mut Vec<Bomb>, enemies: &mut Vec<Enemy>) {
        self.bound(map);
        self.collide_map(map, direction);
        for bomb in bombs {
            if bomb.owner_id == self.id && !bomb.collision_enabled {
                continue;
            }
            if self.resolve_collision_with(bomb.position, bomb.get_size(), direction)
                && self.can_kick_bomb
                && bomb.state == BombState::Planted
            {
                bomb.state = BombState::Sliding(direction);
            }
        }

        for enemy in enemies {
            if self.resolve_collision_with(enemy.position, enemy.get_size(), direction) {
                self.kill();
                break;
            }
        }
    }

    pub fn create_bomb(&mut self, resources: &Resources, bombs: &Vec<Bomb>) -> Option<Bomb> {
        if self.bombs_remaining == 0 {
            return None;
        }
        for bomb in bombs {
            if bomb.owner_id == self.id && !bomb.collision_enabled {
                return None;
            }
        }
        //TODO:
        // check position doesn't have another player
        self.bombs_remaining -= 1;
        Some(Bomb::new(
            self.id,
            self.position.x as usize,
            self.position.y as usize,
            self.power_level,
            resources,
        ))
    }

    pub fn player_move(&mut self, input: Input, delta: f32, map: &Map, bombs: &mut Vec<Bomb>, enemies: &mut Vec<Enemy>) {
        let mut motion = input.as_vec2()
            * delta
            * PLAYER_SPEEDS[(self.speed_level as usize).min(PLAYER_SPEEDS.len() - 1)];

        let mut dist: f32;
        while motion.x != 0.0 || motion.y != 0.0 {
            // X tick
            if motion.x != 0.0 {
                dist = motion.x.abs().min(1.0);
                if motion.x > 0.0 {
                    self.direction = Direction::Right;
                    motion.x -= dist;
                    self.position.x += dist;
                } else {
                    self.direction = Direction::Left;
                    motion.x += dist;
                    self.position.x -= dist;
                }
                self.handle_collisions(map, self.direction, bombs, enemies);
            }
            if motion.y != 0.0 {
                dist = motion.y.abs().min(1.0);
                if motion.y > 0.0 {
                    self.direction = Direction::Down;
                    motion.y -= dist;
                    self.position.y += dist;
                } else {
                    self.direction = Direction::Up;
                    motion.y += dist;
                    self.position.y -= dist;
                }
                self.handle_collisions(map, self.direction, bombs, enemies);
            }
        }
        match &mut self.object {
            None => (),
            Some(obj) => {
                obj.transform.translation = Vec3::new(self.position.x, 0.0, self.position.y);
                let (x, y) = input.as_vec2().into();
                if x != 0.0 || y != 0.0 {
                    obj.transform.rotation.y = x.atan2(y);
                }
            }
        }
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.object = None;
    }
}

impl Collision for Player {
    fn get_pos(&self) -> Vec2 {
        self.position
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos;
    }

    fn get_size(&self) -> f32 {
        PLAYER_RADIUS
    }
}

pub trait Alive {
    fn alive(&mut self) -> impl Iterator<Item = &'_ mut Player>;
}

impl Alive for Vec<Player> {
    fn alive(&mut self) -> impl Iterator<Item = &'_ mut Player> {
        self.iter_mut().filter(|p| p.alive)
    }
}
