use std::fs::OpenOptions;

use serde::{Deserialize, Serialize};
use winit::keyboard::KeyCode;

use crate::input::{input::Binds, input_name::InputName};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub framerate_limit: u32,
    pub volume_music: f32,
    pub volume_sfx: f32,
    pub binds: Vec<Binds>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut p1_binds: Binds = [KeyCode::F35; 5];
        p1_binds[InputName::Up as usize] = KeyCode::KeyW;
        p1_binds[InputName::Down as usize] = KeyCode::KeyS;
        p1_binds[InputName::Left as usize] = KeyCode::KeyA;
        p1_binds[InputName::Right as usize] = KeyCode::KeyD;
        p1_binds[InputName::Bomb as usize] = KeyCode::Space;

        let mut p2_binds: Binds = [KeyCode::F35; 5];
        p2_binds[InputName::Up as usize] = KeyCode::ArrowUp;
        p2_binds[InputName::Down as usize] = KeyCode::ArrowDown;
        p2_binds[InputName::Left as usize] = KeyCode::ArrowLeft;
        p2_binds[InputName::Right as usize] = KeyCode::ArrowRight;
        p2_binds[InputName::Bomb as usize] = KeyCode::ShiftRight;

        Self {
            resolution: (800, 600),
            fullscreen: false,
            framerate_limit: 60,
            volume_music: 100.0,
            volume_sfx: 100.0,
            binds: vec![p1_binds, p2_binds],
        }
    }
}

const SETTINGS_PATH: &str = "./settings.bomb";

impl Settings {
    pub fn save(&self) {
        let settings_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(SETTINGS_PATH);
        if settings_file.is_err() {
            println!("save: failed to open {}", SETTINGS_PATH);
            return;
        }

        let settings_file = settings_file.unwrap();

        if serde_cbor::to_writer(settings_file, self).is_err() {
            println!("save: couldn't write settings to {}", SETTINGS_PATH);
        }
    }

    pub fn load_settings() -> Self {
        let settings_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(SETTINGS_PATH);

        if settings_file.is_err() {
            println!(
                "couldn't read or create file \"{}\", loading default settings",
                SETTINGS_PATH
            );
            return Self::default();
        }
        let load: Result<Settings, serde_cbor::Error> =
            serde_cbor::from_reader(settings_file.unwrap());
        if load.is_err() {
            println!("file {} is invalid, saving defaults", SETTINGS_PATH);
            let default = Self::default();
            default.save();
            return default;
        }
        return load.unwrap();
    }
}
