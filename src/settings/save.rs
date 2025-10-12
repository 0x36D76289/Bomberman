use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::path::Path;

const SAVE_PATH: &str = "./save.bomb";

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SaveState {
    pub level: u32,
    pub lives: u32,
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

impl SaveState {
    pub fn save(&self) {
        let save_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(SAVE_PATH);

        if let Ok(file) = save_file {
            if let Err(e) = serde_cbor::to_writer(file, self) {
                println!("Error: couldn't write to save file {}: {}", SAVE_PATH, e);
            }
        } else {
            println!("Error: failed to open or create save file {}", SAVE_PATH);
        }
    }

    pub fn load() -> Self {
        if !save_file_exists() {
            println!("No save file found. Starting a new game.");
            return Self::default();
        }

        match File::open(SAVE_PATH) {
            Ok(file) => match serde_cbor::from_reader(file) {
                Ok(state) => {
                    println!("Save file loaded successfully.");
                    state
                }
                Err(e) => {
                    println!(
                        "Error: Save file is corrupt, starting a new game. Error: {}",
                        e
                    );
                    Self::default()
                }
            },
            Err(_) => {
                println!("Error: Could not open save file. Starting a new game.");
                Self::default()
            }
        }
    }
}

pub fn save_file_exists() -> bool {
    Path::new(SAVE_PATH).exists()
}
