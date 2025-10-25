use glam::{Vec2, bool};

use crate::game::{
    direction::Direction,
    map::{map::Map, map_element::MapElement},
};

/// The width of the invisible layer around collision, avoid jittering
const PUSHBACK: f32 = 0.01;

/// The [Collision] trait makes an object collidable with and lets you easily test and resolve
/// collisions between game objects
pub trait Collision {
    /// Get the game object's positon
    fn get_pos(&self) -> Vec2;
    /// Set a game object's position
    fn set_pos(&mut self, pos: Vec2);
    /// Get a game object's radius (collision are boxes)
    fn get_size(&self) -> f32;

    /// Clamps a game object's position to be within the map
    fn bound(&mut self, map: &Map) {
        let mut pos = self.get_pos();
        pos.x = pos
            .x
            .clamp(self.get_size(), map.width as f32 - self.get_size());
        pos.y = pos
            .y
            .clamp(self.get_size(), map.height as f32 - self.get_size());
        self.set_pos(pos);
    }

    fn is_colliding_with(&self, pos: Vec2, radius: f32) -> bool {
        ((self.get_pos().x - pos.x).abs() < (self.get_size() + radius))
            && ((self.get_pos().y - pos.y).abs() < (self.get_size() + radius))
    }

    /// Resolves the collision between 2 game objects
    fn resolve_collision_with(&mut self, pos: Vec2, mut radius: f32, direction: Direction) -> bool {
        if !self.is_colliding_with(pos, radius) {
            return false;
        }
        radius += PUSHBACK;
        match direction {
            Direction::Up => self.set_pos(Vec2 {
                x: self.get_pos().x,
                y: pos.y + radius + self.get_size(),
            }),

            Direction::Down => self.set_pos(Vec2 {
                x: self.get_pos().x,
                y: pos.y - radius - self.get_size(),
            }),

            Direction::Left => self.set_pos(Vec2 {
                x: pos.x + radius + self.get_size(),
                y: self.get_pos().y,
            }),

            Direction::Right => self.set_pos(Vec2 {
                x: pos.x - radius - self.get_size(),
                y: self.get_pos().y,
            }),
        };
        true
    }

    /// Checks if a game object is currently colliding with the [Map]
    fn does_map_collide(map: &Map, x: f32, y: f32) -> bool {
        if x < 0.0 || y < 0.0 {
            return true;
        }
        if (x as usize >= map.width) || (y as usize >= map.height) {
            return true;
        }
        match map.get_elem(x as usize, y as usize) {
            MapElement::Empty | MapElement::Exit(_) => false,
            MapElement::Breakable(_) => true,
            MapElement::Unbreakable(_) => true,
        }
    }

    /// Tests if an object is colliding with the map and makes it move back to an empty area
    /// returns True if a collision was found/resolved
    fn collide_map(&mut self, map: &Map, direction: Direction) -> bool {
        let mut ret = false;
        for y in -1..=1 {
            for x in -1..=1 {
                if Self::does_map_collide(
                    map,
                    self.get_pos().x + x as f32,
                    self.get_pos().y + y as f32,
                ) {
                    if self.resolve_collision_with(
                        Vec2 {
                            x: (self.get_pos().x + x as f32) as usize as f32 + 0.5,
                            y: (self.get_pos().y + y as f32) as usize as f32 + 0.5,
                        },
                        0.5,
                        direction,
                    ) {
                        ret = true;
                    }
                }
            }
        }
        ret
    }
}
