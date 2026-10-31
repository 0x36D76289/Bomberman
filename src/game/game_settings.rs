use crate::game::bot::BotLevel;

#[derive(Debug, Clone)]
pub struct GameSettings {
    pub nb_humans: u32,
    pub nb_bots: u32,
    pub bot_level: BotLevel,
    pub teams: Option<Vec<u32>>,
}
