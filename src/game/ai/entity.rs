use glam::Vec2;

use crate::game::game_state::GameState;

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Player(usize),
    Bomb(usize),
    Explosion,
    Powerup(Vec2),
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub position: Vec2,
    pub entity_type: EntityType,
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.entity_type == other.entity_type
    }
}

impl Entity {
    pub fn new(entity_type: EntityType) -> Self {
        Entity {
            position: Vec2::new(0.0, 0.0),
            entity_type,
        }
    }

    pub fn get_id(self) -> usize {
        match self.entity_type {
            EntityType::Player(id) => id,
            EntityType::Bomb(id) => id,
            EntityType::Explosion => 0,
            EntityType::Powerup(Vec2) => 0,
        }
    }

    pub fn get_players_from_list(entities: &[Entity]) -> Vec<usize> {
        entities
            .iter()
            .flat_map(|entity| match entity.entity_type {
                EntityType::Player(id) => Some(id),
                _ => None,
            })
            .collect()
    }
}
