use crate::{
    game::{
        bomb::{BOMB_RADIUS, Bomb, BombState},
        collision::Collision,
        map::{map::Map, map_element::MapElement},
        resources::{ResourceName, Resources},
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
            object: Some(Self::create_object(resources, position, direction)),
        }
    }

    fn create_object(resources: &Resources, position: Vec2, direction: Direction) -> Object {
        let dir_vec = direction.to_vec2();
        Object {
            model: resources.models[&ResourceName::Player].clone(),
            texture: Some(resources.textures_index[&ResourceName::Player]),
            color: Vec3::ONE,
            transform: Transform {
                translation: Vec3::new(position.x, 0.0, position.y),
                scale: Vec3::splat(0.35),
                rotation: Vec3::new(0.0, dir_vec.x.atan2(dir_vec.y), 0.0),
            },
        }
    }

    pub fn make_op(&mut self, resources: &Resources) {
        self.object = Some(Self::create_object(
            resources,
            self.position,
            self.direction,
        ));
        self.alive = true;
        self.power_level = 10;
        self.speed_level = 4;
        self.bombs_remaining = 100;
        self.can_kick_bomb = true;
    }

    fn handle_collisions(&mut self, map: &Map, direction: Direction, bombs: &mut Vec<Bomb>) {
        self.bound(map);
        self.collide_map(map, direction);
        for bomb in bombs {
            if bomb.owner_id == self.id && !bomb.collision_enabled {
                continue;
            }
            if self.resolve_collision_with(bomb.position, BOMB_RADIUS, direction)
                && self.can_kick_bomb
                && bomb.state == BombState::Planted
            {
                bomb.state = BombState::Sliding(direction);
            }
        }
    }

    pub fn create_bomb(&mut self, resources: &Resources, bombs: &Vec<Bomb>) -> Option<Bomb> {
        if self.bombs_remaining == 0 {
            return None;
        }

        let target_x = self.position.x as usize;
        let target_y = self.position.y as usize;

        for bomb in bombs {
            if bomb.owner_id == self.id && !bomb.collision_enabled {
                return None;
            }
            if bomb.is_colliding_with(
                Vec2 {
                    x: target_x as f32,
                    y: target_y as f32,
                },
                BOMB_RADIUS,
            ) {
                return None;
            }
        }

        //TODO:
        // check position doesn't have another player / enemy

        self.bombs_remaining -= 1;
        Some(Bomb::new(
            self.id,
            target_x,
            target_y,
            self.power_level,
            resources,
        ))
    }

    fn assist_input(&self, input: Input, map: &Map) -> Vec2 {
        let ret = input.as_vec2();
        if ret.y == 0.0 && ret.x != 0.0 {
            if *map.get_elem_pos(self.position + ret) == MapElement::Empty {
                if self.position.y % 1.0 > 0.60 {
                    return ret + Vec2 { x: 0.0, y: -1.0 };
                }
                if self.position.y % 1.0 < 0.40 {
                    return ret + Vec2 { x: 0.0, y: 1.0 };
                }
            }
        }
        if ret.x == 0.0 && ret.y != 0.0 {
            if *map.get_elem_pos(self.position + ret) == MapElement::Empty {
                if self.position.x % 1.0 > 0.60 {
                    return ret + Vec2 { y: 0.0, x: -1.0 };
                }
                if self.position.x % 1.0 < 0.40 {
                    return ret + Vec2 { y: 0.0, x: 1.0 };
                }
            }
        }
        ret
    }

    pub fn player_move(&mut self, input: Input, delta: f32, map: &Map, bombs: &mut Vec<Bomb>) {
        if !self.alive {
            return;
        }
        let mut motion = self.assist_input(input, map)
            * delta
            * PLAYER_SPEEDS[(self.speed_level as usize).min(PLAYER_SPEEDS.len() - 1)];

        let mut dist: f32;
        // INFO: The right thing to do would be find the distance to the nearest block center...
        // but I'll just iterate 0.2 at a time because performance is plenty
        while motion.x != 0.0 || motion.y != 0.0 {
            // X tick
            if motion.x != 0.0 {
                dist = motion.x.abs().min(0.2);
                if motion.x > 0.0 {
                    self.direction = Direction::Right;
                    motion.x -= dist;
                    self.position.x += dist;
                } else {
                    self.direction = Direction::Left;
                    motion.x += dist;
                    self.position.x -= dist;
                }
                self.handle_collisions(map, self.direction, bombs);
            }
            if motion.y != 0.0 {
                dist = motion.y.abs().min(0.2);
                if motion.y > 0.0 {
                    self.direction = Direction::Down;
                    motion.y -= dist;
                    self.position.y += dist;
                } else {
                    self.direction = Direction::Up;
                    motion.y += dist;
                    self.position.y -= dist;
                }
                self.handle_collisions(map, self.direction, bombs);
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

    pub fn respawn(&mut self, position: Vec2, resources: &Resources) {
        self.alive = true;
        self.position = position;
        self.object = Some(Self::create_object(resources, position, self.direction));
        self.power_level = 2;
        self.speed_level = 0;
        self.bombs_remaining = 1;
        self.can_kick_bomb = false;
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
