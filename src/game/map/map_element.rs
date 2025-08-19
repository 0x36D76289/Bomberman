use rand::random_range;

use crate::{game::direction::Direction, graphics::object::Object};

#[derive(Clone, Debug)]
pub enum MapElement {
    Empty,
    SpawnPoint(Direction),
    Breakable(Object),
    Unbreakable(Object),
}

impl MapElement {
    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn value(&self) -> char {
        match *self {
            MapElement::Empty => ' ',
            MapElement::SpawnPoint(_) => '*',
            MapElement::Breakable(_) => '#',
            MapElement::Unbreakable(_) => 'X',
        }
    }

    pub fn random_spawn_point() -> Self {
        Self::SpawnPoint(match random_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        })
    }
}
