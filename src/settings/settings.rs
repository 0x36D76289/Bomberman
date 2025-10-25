use std::fs::{self, OpenOptions};
use std::path::Path;

use serde::{Deserialize, Serialize};
use winit::keyboard::KeyCode;

use crate::settings::save::SaveState;
use crate::{
    input::{
        event::InputEvent,
        input::{Binds, default_binds},
        input_name::InputName,
    },
    settings::path::get_settings_path,
};

/// The layout of the entire save file
#[derive(Serialize, Deserialize)]
pub struct Settings {
    /// The resolution of the display in pixel as (x, y)
    pub resolution: (u32, u32),
    /// If scaling should use nearest neighbor or bilinear TODO:
    pub filtering: bool,
    /// Should the window be displayed in fullscreen
    pub fullscreen: bool,
    /// what is the framerate limit in fps, -1 if none, 0 for vsync
    pub framerate_limit: i32,
    /// Music Volume, between 0 and 1
    pub volume_music: f32,
    /// SFX Volume, between 0 and 1
    pub volume_sfx: f32,
    /// Each member of the vector is a player, the content of the element are in order of InputName
    pub binds: Vec<Binds>,
    /// The current save file for single player
    pub single_player_save: SaveState,
}

impl Default for Settings {
    fn default() -> Self {
        let mut p1_binds: Binds = default_binds();
        p1_binds[InputName::Up as usize] = InputEvent::from_keycode(KeyCode::KeyW);
        p1_binds[InputName::Down as usize] = InputEvent::from_keycode(KeyCode::KeyS);
        p1_binds[InputName::Left as usize] = InputEvent::from_keycode(KeyCode::KeyA);
        p1_binds[InputName::Right as usize] = InputEvent::from_keycode(KeyCode::KeyD);
        p1_binds[InputName::Bomb as usize] = InputEvent::from_keycode(KeyCode::Space);
        p1_binds[InputName::Back as usize] = InputEvent::from_keycode(KeyCode::Escape);

        let mut p2_binds: Binds = default_binds();
        p2_binds[InputName::Up as usize] = InputEvent::from_keycode(KeyCode::ArrowUp);
        p2_binds[InputName::Down as usize] = InputEvent::from_keycode(KeyCode::ArrowDown);
        p2_binds[InputName::Left as usize] = InputEvent::from_keycode(KeyCode::ArrowLeft);
        p2_binds[InputName::Right as usize] = InputEvent::from_keycode(KeyCode::ArrowRight);
        p2_binds[InputName::Bomb as usize] = InputEvent::from_keycode(KeyCode::ShiftRight);
        p2_binds[InputName::Back as usize] = InputEvent::from_keycode(KeyCode::Numpad0);

        Self {
            resolution: (800, 600),
            filtering: false,
            fullscreen: false,
            framerate_limit: 60,
            volume_music: 1.0,
            volume_sfx: 1.0,
            binds: vec![p1_binds, p2_binds],
            single_player_save: SaveState::default(),
        }
    }
}

impl Settings {
    /// Saves the settings file to the settings path using [get_settings_path()]
    pub fn save(&self) {
        let path_str = get_settings_path();
        let path = Path::new(&path_str);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    println!("Error creating settings directory: {}", e);
                    return;
                }
            }
        }

        let settings_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path);

        if let Ok(file) = settings_file {
            if let Err(e) = serde_cbor::to_writer(file, self) {
                println!("save: couldn't write settings to {}: {}", path_str, e);
            }
        } else {
            println!("save: failed to open {}", path_str);
        }
    }

    /// Returns the [Settings] struct from the serialized file at [get_settings_path()]
    pub fn load_settings() -> Self {
        let settings_file = OpenOptions::new().read(true).open(get_settings_path());

        if settings_file.is_err() {
            println!(
                "couldn't read file \"{}\", loading default settings",
                get_settings_path()
            );
            return Self::default();
        }
        let load: Result<Settings, serde_cbor::Error> =
            serde_cbor::from_reader(settings_file.unwrap());
        if load.is_err() {
            println!("file {} is invalid, saving defaults", get_settings_path());
            let default = Self::default();
            default.save();
            return default;
        }
        load.unwrap()
    }
}
