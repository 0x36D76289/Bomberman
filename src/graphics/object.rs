use crate::{game::resources::{ResourceName, Resources}, graphics::{StaticMesh, mesh::Mesh, transform::Transform}};
use glam::Vec3;
use std::sync::Arc;

pub type TextureIndex = i32;

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub meshes: Vec<Mesh>,
    pub textures: Vec<Option<TextureIndex>>,
    pub transform: Transform,
    pub color: Vec3,
}

impl Object {
    pub fn from_resource(resource_name: ResourceName, resources: &Resources) -> Self {
        let meshes = resources.models[&resource_name].clone();
        let textures = {
            if !resources.textures_index.contains_key(&resource_name) {
                vec![None; meshes.len()]
            } else {
                resources
                    .textures_index[&resource_name]
                    .iter()
                    .map(|i| if *i == -1 {None} else {Some(*i)})
                    .collect()
            }
        };

        Self {
            meshes: meshes,
            textures: textures,
            transform: Transform::default(),
            color: Vec3::ONE
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}
