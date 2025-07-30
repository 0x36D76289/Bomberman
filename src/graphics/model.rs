use crate::graphics::MyVertex;
use std::{collections::HashMap, error::Error, fs::File, io::{BufReader, Cursor}, sync::Arc};
use tobj::LoadError;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

#[derive(Debug, Clone)]
pub struct Model {
    pub vertex_buffer: Subbuffer<[MyVertex]>,
    pub index_buffer: Subbuffer<[u32]>,
}

#[macro_export]
macro_rules! load_model {
    ($path:literal, $allocator:expr) => {
        {
            let file = include_bytes!($path);
            let mut cursor: Cursor<&[u8]> = Cursor::new(file);
            Arc::new(Model::load(&mut cursor, ($allocator).clone())?)
        }
    };
}

impl Model {
    // pub fn from_path(
    //     path: &str,
    //     memory_allocator: Arc<StandardMemoryAllocator>,
    // ) -> Result<Self, Box<dyn Error>> {
    //     let (models, _) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;
    //     Model::load(models, memory_allocator, true)
    // }

    // pub fn from_path_inverse_y(
    //     path: &str,
    //     memory_allocator: Arc<StandardMemoryAllocator>,
    // ) -> Result<Self, Box<dyn Error>> {
    //     let (models, _) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;
    //     Model::load(models, memory_allocator, false)
    // }

    // pub fn from_buf(
    //     bytes: &mut Cursor<&[u8]>,
    //     memory_allocator: Arc<StandardMemoryAllocator>
    // ) -> Result<Self, Box<dyn Error>> {
    //     let (models, _) = tobj::load_obj_buf(
    //         bytes,
    //         &tobj::GPU_LOAD_OPTIONS,
    //         |p| { Err(LoadError::OpenFileFailed) }
    //     )?;
    //     Model::load(models, memory_allocator, true)
    // }

    // pub fn from_buf_inverse_y(
    //     bytes: &mut Cursor<&[u8]>,
    //     memory_allocator: Arc<StandardMemoryAllocator>
    // ) -> Result<Self, Box<dyn Error>> {
    //     let (models, _) = tobj::load_obj_buf(
    //         bytes,
    //         &tobj::GPU_LOAD_OPTIONS,
    //         |p| { Err(LoadError::OpenFileFailed) }
    //     )?;
    //     Model::load(models, memory_allocator, false)
    // }

    pub fn load(
        bytes: &mut Cursor<&[u8]>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self, Box<dyn Error>> {
        let (models, _) = tobj::load_obj_buf(
            bytes,
            &tobj::GPU_LOAD_OPTIONS,
            |p| { Err(LoadError::OpenFileFailed) }
        )?;

        let mut unique_vertices: HashMap<MyVertex, u32> = HashMap::new();
        let mut vertices = vec![MyVertex::default()];
        let mut indices = Vec::new();

        for model in models {
            for i in model.mesh.indices {
                let mut vertex = MyVertex::default();
                let i = i as usize;

                vertex.position = [
                    model.mesh.positions[i * 3],
                    -model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                ];

                if !model.mesh.normals.is_empty() {
                    vertex.normal = [
                        model.mesh.normals[i * 3],
                        -model.mesh.normals[i * 3 + 1],
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

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )?;

        let index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )?;

        Ok(Self {
            vertex_buffer,
            index_buffer,
        })
    }

    pub fn empty(memory_allocator: Arc<StandardMemoryAllocator>) -> Result<Self, Box<dyn Error>> {
        let vertices: Vec<MyVertex> = Vec::new();
        let indices: Vec<u32> = Vec::new();

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )?;

        let index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )?;

        Ok(Self {
            vertex_buffer,
            index_buffer,
        })
    }
}
