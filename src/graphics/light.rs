use glam::{Vec3, Vec4};

use crate::graphics::Transform;

#[derive(Debug, Default)]
pub struct Light {
    pub transform: Transform,
    pub color: Vec4,
}
