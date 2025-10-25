use glam::{USizeVec2, Vec2, Vec3, usize};
use rand::random_range;

use crate::{
    audio::{AudioManager, SoundEffect},
    game::{
        collision::Collision,
        player::Player,
        resources::{ResourceName, Resources},
    },
    graphics::{object::Object, transform::Transform},
};

/// The various types of [PowerUp]s, they each provide a different bonus
#[derive(Debug, Clone)]
pub enum PowerUpType {
    /// The [Speed](PowerUpType::Speed) [PowerUp] increases the [Player]'s speed
    Speed,
    /// The [Power](PowerUpType::Power) [PowerUp] increases how far a [Player]'s [Bomb]s can explode
    Power,
    /// The [Bomb](PowerUpType::Bomb) [PowerUp] increases how many [Bomb]s a [Player] can
    /// deploy at the same time
    Bomb,
    /// The [Slide](PowerUpType::Slide) [PowerUp] lets a [Player] push [Bomb]s by walking into them
    Slide,
}

impl PowerUpType {
    /// Used when a [Player] obtains a [PowerUp]
    fn apply(&self) -> impl Fn(&mut Player) {
        match self {
            PowerUpType::Speed => |p: &mut Player| p.speed_level += 1,
            PowerUpType::Power => |p: &mut Player| p.power_level += 1,
            PowerUpType::Bomb => |p: &mut Player| p.bombs_remaining += 1,
            PowerUpType::Slide => |p: &mut Player| p.can_kick_bomb = true,
        }
    }
}

/// A [PowerUp] is a bonus a [Player] can pick up to obtain an advantage
#[derive(Debug, Clone)]
pub struct PowerUp {
    /// The type of bonus the [Player] will obtain
    pub power_up_type: PowerUpType,
    /// The [PowerUp]'s 3d model
    pub object: Object,
    /// The [PowerUp]'s position on the board
    pub pos: USizeVec2,
    /// If the [PowerUp] should disappear at the end of the current tick
    pub despawn: bool,
}

impl PowerUp {
    pub fn get_size(&self) -> f32 {
        0.4
    }

    /// The [PowerUp]'s tick function finds all colliding players and applies the correct effect onto them
    pub fn tick(&mut self, players: &mut Vec<Player>, audio_manager: &mut AudioManager) {
        for player in players {
            if player.is_colliding_with(
                Vec2 {
                    x: self.pos.x as f32 + 0.5,
                    y: self.pos.y as f32 + 0.5,
                },
                self.get_size(),
            ) {
                audio_manager.play_sound_effect(SoundEffect::BonusPickup);
                self.power_up_type.apply()(player);
                self.despawn = true;
            }
        }
    }
}

impl PowerUp {
    /// The main [PowerUp] constructor
    pub fn new(y: usize, x: usize, resources: &Resources) -> Self {
        let (power_up_type, model, texture) = match random_range(0..=3) {
            0 => (
                PowerUpType::Speed,
                resources.models[&ResourceName::PowerSpeed].clone(),
                resources.textures_index[&ResourceName::PowerSpeed],
            ),
            1 => (
                PowerUpType::Power,
                resources.models[&ResourceName::PowerPower].clone(),
                resources.textures_index[&ResourceName::PowerPower],
            ),
            2 => (
                PowerUpType::Bomb,
                resources.models[&ResourceName::PowerBomb].clone(),
                resources.textures_index[&ResourceName::PowerBomb],
            ),
            _ => (
                PowerUpType::Slide,
                resources.models[&ResourceName::PowerSlide].clone(),
                resources.textures_index[&ResourceName::PowerSlide],
            ),
        };

        Self {
            power_up_type,
            object: Object {
                model,
                texture: Some(texture),
                transform: Transform {
                    translation: Vec3 {
                        x: x as f32 + 0.5,
                        y: -0.1,
                        z: y as f32 + 0.5,
                    },
                    scale: Vec3::splat(0.9),
                    rotation: Vec3::ZERO,
                },
                color: Vec3::ONE,
            },
            pos: USizeVec2 { x, y },
            despawn: false,
        }
    }
}
