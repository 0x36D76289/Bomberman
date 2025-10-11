use glam::Vec2;

use crate::game::game_state::GameState;

#[derive(Debug, Clone)]
pub enum EntityType {
    Player(i32),
    Bomb(usize),
    Explosion,
    Powerup(Vec2),
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub position: Vec2,
    pub entity_type: EntityType,
}

impl Entity {}
