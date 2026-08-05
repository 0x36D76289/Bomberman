use crate::{game::resources::{ResourceName, Resources}, graphics::{StaticMesh, mesh::Mesh, transform::Transform}};
use glam::Vec3;
use std::sync::Arc;

pub type TextureIndex = i32;

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub meshes: Vec<Mesh>,
    pub texture: Option<TextureIndex>,
    pub transform: Transform,
    pub color: Vec3,
}

impl Object {
    pub fn from_resource(resource_name: &str, resources: &Resources) -> Self {
        let meshes = resources.model(resource_name);
        let texture = resources.texture(resource_name);

        Self {
            meshes: meshes,
            texture,
            transform: Transform::default(),
            color: Vec3::ONE
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}
