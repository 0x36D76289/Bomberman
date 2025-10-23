use std::error::Error;

use crate::{game::map::map_settings::MapSettings, settings::settings::Settings};

pub struct GameSettings {
    pub nb_humans: u32,
    pub map_settings: MapSettings,
}

impl GameSettings {
    pub fn default() -> Self {
        Self {
            nb_humans: 1,
            map_settings: MapSettings::default(),
        }
    }

    pub fn new(settings: &Settings, map_settings: MapSettings) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            map_settings,
            nb_humans: settings.binds.len() as u32,
        })
    }

    pub fn set_map(&mut self, map_settings: MapSettings) {
        self.map_settings = map_settings;
    }

    pub fn set_nb_humans(&mut self, nb_humans: u32) {
        self.nb_humans = nb_humans;
    }
}
