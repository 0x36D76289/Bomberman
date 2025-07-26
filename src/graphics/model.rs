use crate::graphics::MyVertex;
use std::{collections::HashMap, error::Error, path::Path};

pub struct Model {
    pub vertices: Vec<MyVertex>,
    pub indices: Vec<u32>,
}

impl Model {
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let (models, _) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;

        let mut unique_vertices: HashMap<MyVertex, u32> = HashMap::new();
        let mut vertices = vec![MyVertex::default()];
        let mut indices = Vec::new();

        for model in models {
            for i in model.mesh.indices {
                let mut vertex = MyVertex::default();
                let i = i as usize;

                vertex.position = [
                    model.mesh.positions[i * 3],
                    model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                ];

                if !model.mesh.normals.is_empty() {
                    vertex.normal = [
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2],
                    ];
                }

                if !model.mesh.texcoords.is_empty() {
                    vertex.uv = [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]];
                }

                if !unique_vertices.contains_key(&vertex) {
                    unique_vertices.insert(vertex, vertices.len() as u32);
                    vertices.push(vertex);
                }
                indices.push(unique_vertices[&vertex]);
            }
        }
        Ok(Self { vertices, indices })
    }
}
