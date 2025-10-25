use serde::{Deserialize, Serialize};

/// The structure containing the singleplayer campaign data
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SaveState {
    /// The current level
    pub level: u32,
    /// The amount of lives remaining
    pub lives: u32,
    // TODO: unimplemented
    /// The score of the ongoing campaign
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
