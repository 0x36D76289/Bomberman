use crate::graphics::{Model, transform::Transform};
use glam::Vec3;
use std::sync::Arc;

pub type TextureIndex = i32;

#[derive(Debug, Clone)]
pub struct Object {
    pub model: Arc<Model>,
    pub texture: Option<TextureIndex>,
    pub transform: Transform,
    pub color: Vec3,
}
