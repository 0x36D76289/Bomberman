use crate::game::player::Powers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum GameDifficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl GameDifficulty {
    pub fn label(self) -> &'static str {
        match self {
            GameDifficulty::Easy => "Facile",
            GameDifficulty::Normal => "Normal",
            GameDifficulty::Hard => "Difficile",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveState {
    pub level: u32,
    pub lives: u32,
    pub score: u32,
    #[serde(default)]
    pub difficulty: GameDifficulty,
    #[serde(default)]
    pub random_layout: bool,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub powers: Powers,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            level: 1,
            lives: 3,
            score: 0,
            difficulty: GameDifficulty::Normal,
            random_layout: false,
            seed: 0,
            powers: Powers::default(),
        }
    }
}

impl SaveState {
    pub fn new_game(difficulty: GameDifficulty, random_layout: bool) -> Self {
        Self {
            difficulty,
            random_layout,
            seed: rand::random(),
            ..Default::default()
        }
    }
}
