use serde::{Deserialize, Serialize};

/// Difficulty selection for the campaign
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GameDifficulty {
    Easy,
    Normal,
    Hard,
}

impl GameDifficulty {
    pub fn label(self) -> &'static str {
        match self {
            GameDifficulty::Easy => "Easy",
            GameDifficulty::Normal => "Normal",
            GameDifficulty::Hard => "Hard",
        }
    }
}

impl Default for GameDifficulty {
    fn default() -> Self {
        GameDifficulty::Normal
    }
}

/// The structure containing the singleplayer campaign data
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SaveState {
    /// The current level
    pub level: u32,
    /// The amount of lives remaining
    pub lives: u32,
    /// The score of the ongoing campaign
    pub score: u32,
    /// The difficulty of the ongoing campaign
    #[serde(default)]
    pub difficulty: GameDifficulty,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            level: 1,
            lives: 3,
            score: 0,
            difficulty: GameDifficulty::Normal,
        }
    }
}
