use super::direction::Direction;
use glam::Vec2;

const PLAYER_RADIUS: f32 = 0.4;

pub struct Player {
    pub id: u32,
    pub position: Vec2,
    pub direction: Direction,
    pub alive: bool,
    pub power_level: u32,
    pub speed: f32,
    pub bombs_remaining: u32,
    pub is_human: bool,
    pub can_kick_bomb: bool,
}
