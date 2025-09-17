use glam::Vec2;
use rand::random_range;

use crate::{game::direction::Direction, graphics::object::Object};

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnPoint {
    pub direction: Direction,
    pub x: i32,
    pub y: i32,
}

impl SpawnPoint {
    pub fn init(x: i32, y: i32) -> Self {
        let direction = match random_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        };
        SpawnPoint { direction, x, y }
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

impl MapElement {
    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn value(&self) -> char {
        match *self {
            MapElement::Empty => ' ',
            MapElement::Breakable(_) => '#',
            MapElement::Unbreakable(_) => 'X',
            MapElement::Exit(_) => 'O',
        }
    }
}
