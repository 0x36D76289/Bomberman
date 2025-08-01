use glam::{Vec3, Vec4};

#[derive(Debug, Default)]
pub struct Light {
    pub position: Vec3,
    pub color: Vec4,
}
