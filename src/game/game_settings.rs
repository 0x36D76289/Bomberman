use crate::game::map::map_settings::MapSettings;

pub struct GameSettings {
    pub nb_humans: u32,
    pub nb_bots: u32,
    pub map_settings: MapSettings,
}

impl GameSettings {
    pub fn default() -> Self {
        Self {
            nb_humans: 1,
            nb_bots: 0,
            map_settings: MapSettings::default(),
        }
    }
}
