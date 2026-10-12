use crate::graphics::{Model, mesh::SkinnedMesh, transform::Transform};
use glam::Vec3;
use std::sync::Arc;

pub type TextureIndex = i32;

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub model: Arc<Model>,
    pub texture: Option<TextureIndex>,
    pub transform: Transform,
    pub color: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedObject {
    pub mesh: SkinnedMesh,
    pub texture: Option<TextureIndex>,
    pub transform: Transform,
    pub color: Vec3,
    pub tint_whites: bool,
}
