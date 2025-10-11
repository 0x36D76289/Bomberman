use glam::Vec2;

pub enum EntityType {
    Player,
    Bomb,
    Explosion,
    Powerup,
}
pub struct Entity {
    pub position: Vec2,
    pub entity_type: EntityType,
}
