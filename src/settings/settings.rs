use crate::game::map::map::LEVEL_COUNT;
use crate::settings::save::SaveState;
use crate::{
    input::{
        event::InputEvent,
        input::{BIND_LEN, Binds, default_binds},
    },
    settings::path::get_settings_path,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use winit::keyboard::KeyCode;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub resolution: (u32, u32),
    pub filtering: bool,
    #[serde(default = "enabled")]
    pub pixel_art: bool,
    pub fullscreen: bool,
    pub framerate_limit: i32,
    pub volume_music: f32,
    pub volume_sfx: f32,
    #[serde(deserialize_with = "padded_binds")]
    pub binds: Vec<Binds>,
    #[serde(default)]
    pub single_player_save: Option<SaveState>,
    #[serde(default = "first_level")]
    pub unlocked_level: u32,
    #[serde(default)]
    pub intro_seen: bool,
}

fn padded_binds<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Vec<Binds>, D::Error> {
    Ok(Vec::<Vec<InputEvent>>::deserialize(de)?
        .into_iter()
        .enumerate()
        .map(|(player, events)| {
            let mut binds = Settings::player_binds(player);
            for (slot, event) in binds.iter_mut().zip(events) {
                *slot = event;
            }
            binds
        })
        .collect())
}

fn first_level() -> u32 {
    1
}

fn enabled() -> bool {
    true
}

fn key_binds(keys: [KeyCode; BIND_LEN]) -> Binds {
    keys.map(InputEvent::from_keycode)
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: (1280, 720),
            filtering: true,
            pixel_art: true,
            fullscreen: false,
            framerate_limit: 0,
            volume_music: 0.7,
            volume_sfx: 0.8,
            binds: vec![
                key_binds([
                    KeyCode::KeyW,
                    KeyCode::KeyS,
                    KeyCode::KeyA,
                    KeyCode::KeyD,
                    KeyCode::Space,
                    KeyCode::Escape,
                    KeyCode::KeyE,
                ]),
                key_binds([
                    KeyCode::ArrowUp,
                    KeyCode::ArrowDown,
                    KeyCode::ArrowLeft,
                    KeyCode::ArrowRight,
                    KeyCode::ShiftRight,
                    KeyCode::Numpad0,
                    KeyCode::ControlRight,
                ]),
            ],
            single_player_save: None,
            unlocked_level: 1,
            intro_seen: false,
        }
    }
}

impl Settings {
    pub fn player_binds(player: usize) -> Binds {
        Self::default()
            .binds
            .get(player)
            .copied()
            .unwrap_or_else(default_binds)
    }

    pub fn save(&self) {
        let path = get_settings_path();
        if let Some(parent) = path.parent()
            && fs::create_dir_all(parent).is_err()
        {
            return;
        }
        if let Ok(file) = File::create(path) {
            let _ = ciborium::into_writer(self, file);
        }
    }

    pub fn load_settings() -> Self {
        let Ok(file) = File::open(get_settings_path()) else {
            return Self::default();
        };
        let Ok(mut settings) = ciborium::from_reader::<Settings, _>(file) else {
            let default = Self::default();
            default.save();
            return default;
        };
        if settings.binds.is_empty() {
            settings.binds = Self::default().binds;
        }
        settings.unlocked_level = settings.unlocked_level.clamp(1, LEVEL_COUNT);
        if let Some(save) = &mut settings.single_player_save {
            save.level = save.level.clamp(1, LEVEL_COUNT);
        }
        settings
    }

    pub fn save_progress(&mut self, save: SaveState) {
        self.unlocked_level = self.unlocked_level.max(save.level);
        self.single_player_save = Some(save);
        self.save();
    }
}
