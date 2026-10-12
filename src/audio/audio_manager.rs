use anyhow::Result;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use std::{collections::HashMap, sync::Arc};

use crate::settings::{assets::read_asset, settings::Settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    BombExplosion,
    PutBomb,
    NoBomb,
    PlayerDeath,
    EnemyHit,
    EnemyDeath,
    BonusPickup,
    WallBreak,
    Kick,
    DoorOpen,
    Alarm,
    MenuMove,
    MenuConfirm,
    MenuBack,
    MenuSwitch,
    LevelStart,
    StageClear,
    GameOver,
    Victory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackgroundMusic {
    Menu,
    Select,
    World1,
    World2,
    World3,
    World4,
    Boss,
    Arena,
    Ending,
}

struct Output {
    device: MixerDeviceSink,
    music_sink: Player,
}

pub struct AudioManager {
    output: Option<Output>,
    sound_effects: HashMap<SoundEffect, Arc<[u8]>>,
    background_music: HashMap<BackgroundMusic, Arc<[u8]>>,
    current_music: Option<BackgroundMusic>,
    sfx_volume: f32,
    music_volume: f32,
}

#[derive(Clone)]
struct SharedBytes(Arc<[u8]>);

impl AsRef<[u8]> for SharedBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AudioManager {
    pub fn new(settings: &Settings) -> Result<Self> {
        let mut sound_effects = HashMap::new();
        for (effect, path) in [
            (SoundEffect::BombExplosion, "sounds/explosion.ogg"),
            (SoundEffect::PutBomb, "sounds/put_bomb.ogg"),
            (SoundEffect::NoBomb, "sounds/no_bomb.ogg"),
            (SoundEffect::PlayerDeath, "sounds/player_death.ogg"),
            (SoundEffect::EnemyHit, "sounds/kick.ogg"),
            (SoundEffect::EnemyDeath, "sounds/enemy_death.ogg"),
            (SoundEffect::BonusPickup, "sounds/bonus.ogg"),
            (SoundEffect::WallBreak, "sounds/wall_break.ogg"),
            (SoundEffect::Kick, "sounds/kick.ogg"),
            (SoundEffect::DoorOpen, "sounds/door.ogg"),
            (SoundEffect::Alarm, "sounds/alarm.ogg"),
            (SoundEffect::MenuMove, "sounds/ui_tap.ogg"),
            (SoundEffect::MenuConfirm, "sounds/ui_click.ogg"),
            (SoundEffect::MenuBack, "sounds/menu_back.ogg"),
            (SoundEffect::MenuSwitch, "sounds/ui_switch.ogg"),
            (SoundEffect::LevelStart, "sounds/level_start.ogg"),
            (SoundEffect::StageClear, "sounds/stage_clear.ogg"),
            (SoundEffect::GameOver, "sounds/game_over.ogg"),
            (SoundEffect::Victory, "sounds/victory.ogg"),
        ] {
            sound_effects.insert(effect, Arc::from(read_asset(path)?));
        }
        let mut background_music = HashMap::new();
        for (music, path) in [
            (BackgroundMusic::Menu, "music/menu.wav"),
            (BackgroundMusic::Select, "music/select.ogg"),
            (BackgroundMusic::World1, "music/world1.ogg"),
            (BackgroundMusic::World2, "music/world2.wav"),
            (BackgroundMusic::World3, "music/world3.ogg"),
            (BackgroundMusic::World4, "music/world4.wav"),
            (BackgroundMusic::Boss, "music/boss.ogg"),
            (BackgroundMusic::Arena, "music/arena.wav"),
            (BackgroundMusic::Ending, "music/ending.wav"),
        ] {
            background_music.insert(music, Arc::from(read_asset(path)?));
        }

        let mut audio_manager = Self {
            output: match DeviceSinkBuilder::open_default_sink() {
                Ok(mut device) => {
                    device.log_on_drop(false);
                    Some(Output {
                        music_sink: Player::connect_new(device.mixer()),
                        device,
                    })
                }
                Err(_) => None,
            },
            sound_effects,
            background_music,
            current_music: None,
            sfx_volume: 1.0,
            music_volume: 1.0,
        };
        audio_manager.set_volume(settings);
        Ok(audio_manager)
    }

    pub fn set_volume(&mut self, settings: &Settings) {
        self.sfx_volume = settings.volume_sfx.clamp(0.0, 1.0);
        self.music_volume = settings.volume_music.clamp(0.0, 1.0);
        if let Some(output) = &self.output {
            output.music_sink.set_volume(self.music_volume);
        }
    }

    pub fn play_sound_effect(&self, effect: SoundEffect) {
        let (Some(output), Some(data)) = (&self.output, self.sound_effects.get(&effect)) else {
            return;
        };
        if self.sfx_volume <= 0.0 {
            return;
        }
        if let Ok(decoder) = Decoder::new(std::io::Cursor::new(SharedBytes(data.clone()))) {
            let sink = Player::connect_new(output.device.mixer());
            sink.set_volume(self.sfx_volume);
            sink.append(decoder);
            sink.detach();
        }
    }

    pub fn play_background_music(&mut self, music: BackgroundMusic) {
        if self.current_music == Some(music) {
            self.resume_background_music();
            return;
        }
        self.stop_background_music();
        self.current_music = Some(music);

        let (Some(output), Some(data)) = (&mut self.output, self.background_music.get(&music))
        else {
            return;
        };
        if let Ok(decoder) = Decoder::new_looped(std::io::Cursor::new(SharedBytes(data.clone()))) {
            output.music_sink.set_volume(self.music_volume);
            output.music_sink.append(decoder);
            output.music_sink.play();
        }
    }

    pub fn stop_background_music(&mut self) {
        if self.current_music.take().is_none() {
            return;
        }
        if let Some(output) = &mut self.output {
            output.music_sink.stop();
            let sink = Player::connect_new(output.device.mixer());
            sink.set_volume(self.music_volume);
            output.music_sink = sink;
        }
    }

    pub fn pause_background_music(&self) {
        if let Some(output) = &self.output {
            output.music_sink.pause();
        }
    }

    pub fn resume_background_music(&self) {
        if let Some(output) = &self.output {
            output.music_sink.play();
        }
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.stop_background_music();
    }
}
