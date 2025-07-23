use glam::Vec2;

pub enum BombState {
    // TODO: Define the states of a bomb
    Planted,
    // Sliding(Direction),
    Exploding,
}

pub struct Bomb {
    pub position: Vec2,
    pub timer: f32,
    pub power: u32,
    pub owner_id: u32,
    pub state: BombState,
}
