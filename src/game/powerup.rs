use crate::{
    audio::{AudioManager, SoundEffect},
    game::{
        collision::Collision,
        grid::{Tile, center_of, tile_transform},
        player::{MAX_SPEED_LEVEL, Player},
        resources::{ResourceName, Resources},
    },
    graphics::object::Object,
};
use glam::Vec3;
use rand::random_range;

pub const MAX_FIRE: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curse {
    Fast,
    Slow,
    ShortFuse,
    WeakFire,
    Diarrhea,
    Constipation,
    Reverse,
}

impl Curse {
    pub const ALL: [Curse; 7] = [
        Curse::Fast,
        Curse::Slow,
        Curse::ShortFuse,
        Curse::WeakFire,
        Curse::Diarrhea,
        Curse::Constipation,
        Curse::Reverse,
    ];

    pub fn random() -> Self {
        Self::ALL[random_range(0..Self::ALL.len())]
    }

    pub fn label(self) -> &'static str {
        match self {
            Curse::Fast => "Turbo",
            Curse::Slow => "Escargot",
            Curse::ShortFuse => "Mèche courte",
            Curse::WeakFire => "Pétard",
            Curse::Diarrhea => "Diarrhée",
            Curse::Constipation => "Constipé",
            Curse::Reverse => "Inversé",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpType {
    BombUp,
    FireUp,
    FullFire,
    SpeedUp,
    Kick,
    Remote,
    BombPass,
    WallPass,
    Vest,
    ExtraLife,
    Skull,
}

impl PowerUpType {
    pub fn name(self) -> &'static str {
        match self {
            PowerUpType::BombUp => "bombs",
            PowerUpType::FireUp => "fire",
            PowerUpType::FullFire => "fullfire",
            PowerUpType::SpeedUp => "speed",
            PowerUpType::Kick => "kick",
            PowerUpType::Remote => "remote",
            PowerUpType::BombPass => "bombpass",
            PowerUpType::WallPass => "wallpass",
            PowerUpType::Vest => "vest",
            PowerUpType::ExtraLife => "life",
            PowerUpType::Skull => "skull",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        [
            PowerUpType::BombUp,
            PowerUpType::FireUp,
            PowerUpType::FullFire,
            PowerUpType::SpeedUp,
            PowerUpType::Kick,
            PowerUpType::Remote,
            PowerUpType::BombPass,
            PowerUpType::WallPass,
            PowerUpType::Vest,
            PowerUpType::ExtraLife,
            PowerUpType::Skull,
        ]
        .into_iter()
        .find(|kind| kind.name() == name.trim())
    }

    pub fn random_battle() -> Self {
        match random_range(0..100) {
            0..30 => PowerUpType::BombUp,
            30..60 => PowerUpType::FireUp,
            60..75 => PowerUpType::SpeedUp,
            75..87 => PowerUpType::Kick,
            87..97 => PowerUpType::Skull,
            _ => PowerUpType::FullFire,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PowerUpType::BombUp => "Bombe +1",
            PowerUpType::FireUp => "Flamme +1",
            PowerUpType::FullFire => "Flamme max",
            PowerUpType::SpeedUp => "Patins",
            PowerUpType::Kick => "Coup de pied",
            PowerUpType::Remote => "Télécommande",
            PowerUpType::BombPass => "Passe-bombes",
            PowerUpType::WallPass => "Passe-murs",
            PowerUpType::Vest => "Gilet",
            PowerUpType::ExtraLife => "1UP",
            PowerUpType::Skull => "Crâne",
        }
    }

    fn look(self) -> (ResourceName, Vec3) {
        match self {
            PowerUpType::BombUp => (ResourceName::PowerBomb, Vec3::ONE),
            PowerUpType::FireUp => (ResourceName::PowerFire, Vec3::ONE),
            PowerUpType::FullFire => (ResourceName::PowerFire, Vec3::new(1.7, 0.35, 0.25)),
            PowerUpType::SpeedUp => (ResourceName::PowerSpeed, Vec3::ONE),
            PowerUpType::Kick => (ResourceName::PowerKick, Vec3::ONE),
            PowerUpType::Remote => (ResourceName::PowerRemote, Vec3::ONE),
            PowerUpType::BombPass => (ResourceName::PowerBomb, Vec3::new(0.45, 0.7, 1.6)),
            PowerUpType::WallPass => (ResourceName::PowerWallPass, Vec3::ONE),
            PowerUpType::Vest => (ResourceName::PowerVest, Vec3::ONE),
            PowerUpType::ExtraLife => (ResourceName::PowerLife, Vec3::ONE),
            PowerUpType::Skull => (ResourceName::PowerSkull, Vec3::new(0.7, 0.35, 1.1)),
        }
    }

    pub fn apply(self, player: &mut Player) {
        let powers = &mut player.powers;
        match self {
            PowerUpType::BombUp => {
                if powers.bombs < 8 {
                    powers.bombs += 1;
                    player.bombs_remaining += 1;
                }
            }
            PowerUpType::FireUp => powers.fire = (powers.fire + 1).min(MAX_FIRE),
            PowerUpType::FullFire => powers.fire = MAX_FIRE,
            PowerUpType::SpeedUp => powers.speed = (powers.speed + 1).min(MAX_SPEED_LEVEL),
            PowerUpType::Kick => powers.kick = true,
            PowerUpType::Remote => powers.remote = true,
            PowerUpType::BombPass => powers.bomb_pass = true,
            PowerUpType::WallPass => powers.wall_pass = true,
            PowerUpType::Vest => player.invincible = 20.0,
            PowerUpType::ExtraLife => (),
            PowerUpType::Skull => player.curse = Some((Curse::random(), 15.0)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PowerUp {
    pub power_up_type: PowerUpType,
    pub object: Object,
    pub tile: Tile,
    pub despawn: bool,
}

impl PowerUp {
    pub fn tick(
        &mut self,
        players: &mut [Player],
        audio_manager: &mut AudioManager,
    ) -> Option<PowerUpType> {
        for player in players.iter_mut().filter(|p| p.alive) {
            if player.is_colliding_with(center_of(self.tile), 0.1) {
                audio_manager.play_sound_effect(SoundEffect::BonusPickup);
                self.power_up_type.apply(player);
                self.despawn = true;
                return Some(self.power_up_type);
            }
        }
        None
    }

    pub fn new(tile: Tile, power_up_type: PowerUpType, resources: &Resources) -> Self {
        let (name, color) = power_up_type.look();
        Self {
            power_up_type,
            object: resources.object(
                name,
                tile_transform(tile.0 as usize, tile.1 as usize, -0.1, 0.9),
                color,
            ),
            tile,
            despawn: false,
        }
    }
}
