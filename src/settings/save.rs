use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SaveState {
    pub level: u32,
    pub lives: u32,
    pub score: u32,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            level: 1,
            lives: 3,
            score: 0,
        }
    }
}
