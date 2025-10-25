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

/// The size of the player, used for every collision
const PLAYER_RADIUS: f32 = 0.4;
/// The speed the player moves at at every level of the powerup
/// It is measured in tiles/second
const PLAYER_SPEEDS: [f32; 4] = [2.0, 3.5, 5.0, 6.0];

/// The [Player](player::Player) represents a human player or a bot in multiplayer games
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Player {
    /// The player's unique identifier, it is also its index in the
    /// [GameState](super::game_state::GameState)'s players vector
    pub id: u32,
    /// The current position of the [Player]
    pub position: Vec2,
    /// The direction the [Player] is facing, used for collisions and to orientate the model
    pub direction: Direction,
    /// Whether the [Player] is alive or dead, dead players don't tick
    pub alive: bool,
    /// How far the [Player]'s [Bomb]s can currently explode
    pub power_level: u8,
    /// What speed value to use from [PLAYER_SPEEDS], can go over the size of the array safely.
    /// Increments with every speed powerup
    pub speed_level: u8,
    /// How many [Bomb]s can the [Player] currently place down. Decrements on every
    /// [Bomb] placed but re-increments on every explosion. Also increases with the bomb powerup
    pub bombs_remaining: u32,
    /// Whether the [Player] is human or bot operated
    pub is_human: bool,
    /// Turns [true] when the [Player] picks up the speed powerup, lets the [Player] push bombs
    /// by colliding with them and pressing in their direction
    pub can_kick_bomb: bool,
    /// The [Player]'s 3d model. [None] if dead
    pub object: Option<Object>,
}

impl Player {
    /// The main [Player] constructor
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

    /// Creates the [Player]'s 3d model
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

    /// Debug utility to make testing easier
    /// Ressucitates the [Player] and gives it every powerup
    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn make_op(&mut self, resources: &Resources) {
        self.object = Some(Self::create_object(
            resources,
            self.position,
            self.direction,
        ));
        self.alive = true;
        self.power_level = 10;
        self.speed_level = PLAYER_SPEEDS.len() as u8;
        self.bombs_remaining = 100;
        self.can_kick_bomb = true;
    }

    /// Collides the [Player] with every object, resolves collisions, and starts expected events
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

    /// Places a [Bomb] if possible and returns it and updates the [Player]'s [bombs remaining](Player::bombs_remaining) value. Returns [None] if creation failed
    pub fn create_bomb(
        &mut self,
        resources: &Resources,
        bombs: &Vec<Bomb>,
        player_positions: &[(u32, Vec2)],
    ) -> Option<Bomb> {
        if self.bombs_remaining == 0 {
            return None;
        }

        let target_x = self.position.x as usize;
        let target_y = self.position.y as usize;

        let ret_bomb = Bomb::new(self.id, target_x, target_y, self.power_level, resources);

        for bomb in bombs {
            if bomb.owner_id == self.id && !bomb.collision_enabled {
                return None;
            }
            if bomb.is_colliding_with(
                Vec2 {
                    x: ret_bomb.position.x as f32,
                    y: ret_bomb.position.y as f32,
                },
                BOMB_RADIUS,
            ) {
                return None;
            }
        }

        for (id, pos) in player_positions {
            if *id == self.id {
                continue;
            }
            if ret_bomb.is_colliding_with(pos.clone(), PLAYER_RADIUS) {
                return None;
            }
        }

        self.bombs_remaining -= 1;
        Some(ret_bomb)
    }

    /// Detects if a [Player] is near a gap and modifies the inputs to guide it towards the gap if the [Player] is trying to get in
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

    /// The [Player] movement function, it is triggered every tick.
    /// It takes in the inputs of the [Player] and makes it move, collide with the [Map], and collide with [Bomb]s
    pub fn player_move(&mut self, input: Input, delta: f32, map: &Map, bombs: &mut Vec<Bomb>) {
        if !self.alive {
            return;
        }
        let mut motion = self.assist_input(input, map)
            * delta
            * PLAYER_SPEEDS[(self.speed_level as usize).min(PLAYER_SPEEDS.len() - 1)];

        let mut dist: f32;
        // PERF: The right thing to do would be find the distance to the nearest block center...
        // but I'll just iterate 0.2 at a time because performance is plenty
        // (also works better at real framerates)
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

    /// Kills the player, effectively removing it
    pub fn kill(&mut self) {
        self.alive = false;
        self.object = None;
    }

    /// Respawns a Player in the singleplayer campaign. Used when colliding with an [Enemy]
    /// if lives are left
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
    /// Returns a list of alive players as mutable
    fn alive(&mut self) -> impl Iterator<Item = &'_ mut Player> {
        self.iter_mut().filter(|p| p.alive)
    }
}
