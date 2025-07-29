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
    pub collision_enabled: bool,
}

const BOMB_TIMER_DEFAULT: f32 = 3.0;
const BOMB_POWER_DEFAULT: u32 = 1;

impl Bomb {
    pub fn new(owner: u32, x: usize, y: usize) -> Self {
        Self {
            position: Vec2 {
                x: x as f32 + 0.5,
                y: y as f32 + 0.5,
            },
            timer: BOMB_TIMER_DEFAULT,
            power: BOMB_POWER_DEFAULT,
            owner_id: owner,
            state: BombState::Planted,
            collision_enabled: false,
        }
    }

    pub fn tick(&self) {
        // if owner not colliding and collision disabled
        // -> enable collision
    }
}
