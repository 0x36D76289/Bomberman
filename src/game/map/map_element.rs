use glam::Vec2;

use crate::{game::direction::Direction, graphics::object::Object};

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnPoint {
    pub direction: Direction,
    pub x: i32,
    pub y: i32,
}

impl SpawnPoint {
    pub fn init(x: i32, y: i32) -> Self {
        SpawnPoint {
            direction: Direction::random(),
            x,
            y,
        }
    }

    pub fn position(&self) -> Vec2 {
        Vec2::new(self.x as f32 + 0.5, self.y as f32 + 0.5)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    Empty,
    Breakable(Object),
    Unbreakable(Object),
    Exit(Object),
}
