use glam::Vec2;
use rand::random_range;

use crate::{game::direction::Direction, graphics::object::Object};

/// A player or bot's spawn location in a multiplayer game
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnPoint {
    /// The direction the player is looking in when spawning
    pub direction: Direction,
    /// The horizontal coordinate of the player spawn
    pub x: i32,
    /// The vertical coordinate of the player spawn
    pub y: i32,
}

impl SpawnPoint {
    /// a [SpawnPoint] constructor for a random direction
    pub fn init(x: i32, y: i32) -> Self {
        let direction = match random_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        };
        SpawnPoint { direction, x, y }
    }

    /// Translate's the [SpawnPoint]'s location to a [Vec2]
    pub fn position(&self) -> Vec2 {
        Vec2::new(self.x as f32 + 0.5, self.y as f32 + 0.5)
    }
}

/// The various values a tile can have
#[derive(Clone, Debug, PartialEq)]
pub enum MapElement {
    /// An empty tile
    Empty,
    /// A breakable box, disappears with a chance of spawning a powerup when exploded
    Breakable(Object),
    /// An unbreakable box, cannot be changed by player action
    Unbreakable(Object),
    /// The exit objects end the level when a player is over it
    Exit(Object),
}

impl MapElement {
    /// a [MapElement]'s character representation for the map's debug display
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
