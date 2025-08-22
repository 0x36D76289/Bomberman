use glam::{USizeVec2, Vec2, Vec3, usize};
use rand::random_range;

use crate::{
    game::{
        collision::Collision,
        player::Player,
        resources::{ResourceName, Resources},
    },
    graphics::{
        object::{Object, TextureIndex},
        transform::Transform,
    },
};

#[derive(Debug)]
pub enum PowerUpType {
    Speed,
    Power,
    Bomb,
    Slide,
}

impl PowerUpType {
    fn apply(&self) -> impl Fn(&mut Player) {
        match self {
            PowerUpType::Speed => |p: &mut Player| p.speed_level += 1,
            PowerUpType::Power => |p: &mut Player| p.power_level += 1,
            PowerUpType::Bomb => |p: &mut Player| p.bombs_remaining += 1,
            PowerUpType::Slide => |p: &mut Player| p.can_kick_bomb = true,
        }
    }
}

#[derive(Debug)]
pub struct PowerUp {
    pub power_up_type: PowerUpType,
    pub object: Object,
    pub pos: USizeVec2,
    pub despawn: bool,
}

impl PowerUp {
    pub fn get_size(&self) -> f32 {
        0.4
    }

    pub fn tick(&mut self, players: &mut Vec<Player>) {
        for player in players {
            if player.is_colliding_with(
                Vec2 {
                    x: self.pos.x as f32 + 0.5,
                    y: self.pos.y as f32 + 0.5,
                },
                self.get_size(),
            ) {
                self.power_up_type.apply()(player);
                self.despawn = true;
            }
        }
    }
}

impl PowerUp {
    pub fn new(y: usize, x: usize, resources: &Resources) -> Self {
        let (power_up_type, model, texture) = match random_range(0..=3) {
            0 => (
                PowerUpType::Speed,
                resources.models[ResourceName::PowerSpeed as usize].clone(),
                ResourceName::PowerSpeed as TextureIndex,
            ),
            1 => (
                PowerUpType::Power,
                resources.models[ResourceName::PowerPower as usize].clone(),
                ResourceName::PowerPower as TextureIndex,
            ),
            2 => (
                PowerUpType::Bomb,
                resources.models[ResourceName::PowerBomb as usize].clone(),
                ResourceName::PowerBomb as TextureIndex,
            ),
            _ => (
                PowerUpType::Slide,
                resources.models[ResourceName::PowerSlide as usize].clone(),
                ResourceName::PowerSlide as TextureIndex,
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
