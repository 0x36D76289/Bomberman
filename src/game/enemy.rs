use super::direction::Direction;
use glam::Vec2;

pub struct Enemy {
    pub id: u32,
    pub position: Vec2,
    pub direction: Direction,
    // pub ai_type: AIType,
    pub alive: bool,
}
