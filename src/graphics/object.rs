use crate::graphics::{StaticMesh, mesh::Mesh, transform::Transform};
use glam::Vec3;
use std::sync::Arc;

pub type TextureIndex = i32;

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub mesh: Mesh,
    pub texture: Option<TextureIndex>,
    pub transform: Transform,
    pub color: Vec3,
}
