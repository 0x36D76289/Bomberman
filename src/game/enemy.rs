use glam::{Vec2, Vec3};
use rand::prelude::*;

use crate::graphics::{object::Object, transform::Transform};

use super::{
    collision::Collision,
    direction::Direction,
    map::map::Map,
    resources::{ResourceName, Resources},
};

const ENEMY_RADIUS: f32 = 0.4;
const ENEMY_SPEED: f32 = 1.5;

/// The Enemy is the main obstacle of the singleplayer campaign
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Enemy {
    /// The unique id of an [Enemy], it is also its position in the enemies vector
    pub id: u32,
    /// The position of the [Enemy]
    pub position: Vec2,
    /// The [Enemy] always walks in the direction it faces
    pub direction: Direction,
    /// Wether the [Enemy] is alive or not, necessary to preserve the [id](Enemy::id)
    pub alive: bool,
    /// The [Enemy]'s 3d model
    pub object: Option<Object>,
}

impl Enemy {
    /// The [Enemy]'s constructor
    pub fn new(id: u32, position: Vec2, resources: &Resources) -> Self {
        Self {
            id,
            position,
            direction: Direction::Down,
            alive: true,
            object: Some(Self::create_object(resources, position, Direction::Down)),
        }
    }

    /// Creates the 3d model for the [Enemy]
    fn create_object(resources: &Resources, position: Vec2, direction: Direction) -> Object {
        let dir_vec = direction.to_vec2();
        Object {
            model: resources.models[&ResourceName::Player].clone(), // TODO: Using player model for now
            texture: Some(resources.textures_index[&ResourceName::Player]),
            color: Vec3::new(1.0, 0.2, 0.2), // Red tint ?
            transform: Transform {
                translation: Vec3::new(position.x, 0.0, position.y),
                scale: Vec3::splat(0.35),
                rotation: Vec3::new(0.0, dir_vec.x.atan2(dir_vec.y), 0.0),
            },
        }
    }

    /// Disables the [Enemy], effectively removing it from the game
    pub fn kill(&mut self) {
        self.alive = false;
        self.object = None;
    }

    /// The [Enemy]'s tick function runs every tick, simulating all events since last frame
    pub fn tick(
        &mut self,
        delta: f32,
        map: &Map,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
    ) {
        if !self.alive {
            return;
        }

        let motion = self.direction.to_vec2() * delta * ENEMY_SPEED;
        self.position += motion;

        // Check for map collisions
        if self.collide_map(map, self.direction) {
            self.position -= motion; // step back
            let mut rng = rand::rng();
            let directions: Vec<_> = Direction::iterator().collect();
            self.direction = **directions.choose(&mut rng).unwrap();
        }

        // Check for bomb collisions
        for bomb in bombs {
            if self.is_colliding_with(bomb.get_pos(), bomb.get_size()) {
                self.position -= motion; // step back
                let mut rng = rand::rng();
                let directions: Vec<_> = Direction::iterator().collect();
                self.direction = **directions.choose(&mut rng).unwrap();
                break;
            }
        }

        // Check for collisions with other enemies
        for other_enemy in other_enemies {
            if other_enemy.id != self.id && other_enemy.alive {
                if self.is_colliding_with(other_enemy.get_pos(), other_enemy.get_size()) {
                    self.position -= motion; // step back
                    let mut rng = rand::rng();
                    let directions: Vec<_> = Direction::iterator().collect();
                    self.direction = **directions.choose(&mut rng).unwrap();
                    break;
                }
            }
        }

        // Update object visuals
        if let Some(obj) = &mut self.object {
            obj.transform.translation = Vec3::new(self.position.x, 0.0, self.position.y);
            let (x, y) = self.direction.to_vec2().into();
            if x != 0.0 || y != 0.0 {
                obj.transform.rotation.y = x.atan2(y);
            }
        }
    }
}

impl Collision for Enemy {
    fn get_pos(&self) -> Vec2 {
        self.position
    }
    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos;
    }
    fn get_size(&self) -> f32 {
        ENEMY_RADIUS
    }
}
