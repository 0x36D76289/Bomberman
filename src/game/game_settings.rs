use crate::game::map::map_settings::MapSettings;

/// The [GameSettings](game_settings::GameSettings) structure is used to create new [GameStates](game_state::GameState)
pub struct GameSettings {
    pub nb_humans: u32,
    pub nb_bots: u32,
    pub map_settings: MapSettings,
}

/// The default values correspond to what is used in the singleplayer campaign
impl GameSettings {
    pub fn default() -> Self {
        Self {
            nb_humans: 1,
            nb_bots: 0,
            map_settings: MapSettings::default(),
        }
    }
}
