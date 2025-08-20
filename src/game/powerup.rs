use glam::{USizeVec2, Vec3, usize};
use rand::random_range;

use crate::{
    game::resources::{ResourceName, Resources},
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

#[derive(Debug)]
pub struct PowerUp {
    pub power_up_type: PowerUpType,
    pub object: Object,
    pub pos: USizeVec2,
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
        }
    }
}
