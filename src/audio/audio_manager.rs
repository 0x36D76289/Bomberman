use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    BombExplosion,
    PutBomb,
    PlayerDeath,
    PlayerHurt,
    EnemyHit,
    BonusPickup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackgroundMusic {
    Menu,
    Game,
    Boss,
}

pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sfx_sink: Sink,
    music_sink: Sink,
    sound_effects: HashMap<SoundEffect, Vec<u8>>,
    background_music: HashMap<BackgroundMusic, Vec<u8>>,
    master_volume: f32,
    sfx_volume: f32,
    music_volume: f32,
    music_enabled: bool,
    sfx_enabled: bool,
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sfx_sink = Sink::try_new(&stream_handle)?;
        let music_sink = Sink::try_new(&stream_handle)?;

        let mut audio_manager = Self {
            _stream: stream,
            stream_handle,
            sfx_sink,
            music_sink,
            sound_effects: HashMap::new(),
            background_music: HashMap::new(),
            master_volume: 1.0,
            sfx_volume: 0.7,
            music_volume: 0.5,
            music_enabled: true,
            sfx_enabled: true,
        };

        audio_manager.load_assets()?;
        Ok(audio_manager)
    }

    fn load_assets(&mut self) -> Result<()> {
        self.load_sound_effect(
            SoundEffect::BombExplosion,
            "src/assets/sounds/bomb_explosion.ogg",
        )?;
        self.load_sound_effect(SoundEffect::PutBomb, "src/assets/sounds/put_bomb.ogg")?;
        self.load_sound_effect(
            SoundEffect::PlayerDeath,
            "src/assets/sounds/player_death.wav",
        )?;
        self.load_sound_effect(SoundEffect::PlayerHurt, "src/assets/sounds/player_hurt.wav")?;
        self.load_sound_effect(SoundEffect::EnemyHit, "src/assets/sounds/enemy_hit_1.ogg")?;
        self.load_sound_effect(SoundEffect::BonusPickup, "src/assets/sounds/bonus.wav")?;

        // Musique de fond
        self.load_background_music(BackgroundMusic::Menu, "src/assets/sounds/menu_loop.ogg")?;
        self.load_background_music(
            BackgroundMusic::Game,
            "src/assets/sounds/eirik_suhrke-a_new_morning.ogg",
        )?;
        self.load_background_music(BackgroundMusic::Boss, "src/assets/sounds/boss1.ogg")?;

        Ok(())
    }

    fn load_sound_effect(&mut self, effect: SoundEffect, path: &str) -> Result<()> {
        if Path::new(path).exists() {
            let data = std::fs::read(path)?;
            self.sound_effects.insert(effect, data);
            println!("Loaded sound effect: {:?} from {}", effect, path);
        } else {
            println!("Warning: Sound file not found: {}", path);
        }
        Ok(())
    }

    fn load_background_music(&mut self, music: BackgroundMusic, path: &str) -> Result<()> {
        if Path::new(path).exists() {
            let data = std::fs::read(path)?;
            self.background_music.insert(music, data);
            println!("Loaded background music: {:?} from {}", music, path);
        } else {
            println!("Warning: Music file not found: {}", path);
        }
        Ok(())
    }

    pub fn play_sound_effect(&self, effect: SoundEffect) {
        if !self.sfx_enabled {
            return;
        }

        if let Some(data) = self.sound_effects.get(&effect) {
            let cursor = std::io::Cursor::new(data.clone());
            if let Ok(decoder) = Decoder::new(cursor) {
                let volume = self.master_volume * self.sfx_volume;
                let source = decoder.amplify(volume);

                // Créer un nouveau sink pour ce son spécifique
                if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                    sink.append(source);
                    sink.detach(); // Le sink se clear tout seul
                }
            }
        }
    }

    pub fn play_background_music(&mut self, music: BackgroundMusic) {
        if !self.music_enabled {
            return;
        }

        self.stop_background_music();

        if let Some(data) = self.background_music.get(&music) {
            let cursor = std::io::Cursor::new(data.clone());
            if let Ok(decoder) = Decoder::new(cursor) {
                let volume = self.master_volume * self.music_volume;
                let source = decoder.repeat_infinite().amplify(volume);
                self.music_sink.append(source);
            }
        }
    }

    pub fn stop_background_music(&mut self) {
        self.music_sink.stop();
        // Créer un nouveau sink pour la prochaine musique
        if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
            self.music_sink = new_sink;
        }
    }

    pub fn pause_background_music(&self) {
        self.music_sink.pause();
    }

    pub fn resume_background_music(&self) {
        self.music_sink.play();
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        self.update_volumes();
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
        self.update_volumes();
    }

    pub fn toggle_music(&mut self) {
        self.music_enabled = !self.music_enabled;
        if !self.music_enabled {
            self.stop_background_music();
        }
    }

    pub fn toggle_sfx(&mut self) {
        self.sfx_enabled = !self.sfx_enabled;
    }

    pub fn is_music_enabled(&self) -> bool {
        self.music_enabled
    }

    pub fn is_sfx_enabled(&self) -> bool {
        self.sfx_enabled
    }

    pub fn get_master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn get_sfx_volume(&self) -> f32 {
        self.sfx_volume
    }

    pub fn get_music_volume(&self) -> f32 {
        self.music_volume
    }

    fn update_volumes(&self) {
        let music_volume = self.master_volume * self.music_volume;
        self.music_sink.set_volume(music_volume);
    }

    pub fn cleanup(&mut self) {
        self.stop_background_music();
        self.sfx_sink.stop();
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
