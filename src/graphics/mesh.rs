use crate::graphics::{AnimationVertex, GameVertex, animation::{Animation, Animator, Joint, JointTransform, KeyFrame}, renderer::animation_vs::JointsUbo};
use std::{collections::HashMap, error::Error, io::Cursor, println, sync::Arc, time::Duration};
use anyhow::Context;
use glam::{Mat4, Quat, Vec2, Vec3};
use rodio::cpal;
use tobj::LoadError;
use ufbx::{LoadOpts, Node, Scene, SceneRoot, SkinDeformer};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Mesh {
    Static(StaticMesh),
    Animated(AnimatedMesh)
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticMesh {
    pub vertex_buffer: Arc<Subbuffer<[GameVertex]>>,
    pub index_buffer: Arc<Subbuffer<[u32]>>,
}

impl StaticMesh {
    pub fn load_from_obj(
        obj_bytes: &[u8],
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut cursor = Cursor::new(obj_bytes);
        
        let (models, _) = tobj::load_obj_buf(&mut cursor, &tobj::GPU_LOAD_OPTIONS, |_| {
            Err(LoadError::OpenFileFailed)
        })?;
        
        let mut unique_vertices: HashMap<GameVertex, u32> = HashMap::new();
        let mut vertices = vec![GameVertex::default()];
        let mut indices = Vec::new();
        
        for model in models {
            for i in model.mesh.indices {
                let mut vertex = GameVertex::default();
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
                    vertex.uv = [
                        model.mesh.texcoords[i * 2],
                        1.0 - model.mesh.texcoords[i * 2 + 1],
                        ];
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
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedMesh {
    pub vertex_buffer: Arc<Subbuffer<[AnimationVertex]>>,
    pub index_buffer: Arc<Subbuffer<[u32]>>,
    pub root_joint: Joint,
    pub joint_count: u32,
    pub animator: Animator,
}

impl AnimatedMesh {
    // pub fn new(mut root_joint: Joint, joint_count: u32) -> Self {
    //     root_joint.calc_inverse_bind_transform(Mat4::IDENTITY);
    //     Self {
    //         root_joint,
    //         joint_count,
    //         animator: Animator::default()
    //     }
    // }

    pub fn get_joint_tranforms(&self) -> Vec<Mat4> {
        let mut joint_transforms = vec![Mat4::IDENTITY; 100];
        AnimatedMesh::add_joints_to_array(&self.root_joint, &mut joint_transforms);
        joint_transforms
    }

    fn add_joints_to_array(head_joint: &Joint, joint_transforms: &mut Vec<Mat4>) {
        // joint_transforms[head_joint.id as usize] = head_joint.animated_transform;
        // for joint in head_joint.children.iter() {
        //     AnimatedMesh::add_joints_to_array(joint, joint_transforms);
        // }

        let id = head_joint.id as usize;
        
        if id < joint_transforms.len() {
            joint_transforms[id] = head_joint.animated_transform;
        }

        for child_joint in head_joint.children.iter() {
            AnimatedMesh::add_joints_to_array(child_joint, joint_transforms);
        }
    }

    pub fn joint_transforms_to_ubo_buffer(&self, buffer: &Subbuffer<JointsUbo>) {
        let mut buffer_write = buffer.write().unwrap();
        let joint_transforms = self.get_joint_tranforms();

        for i in 0..100 {
            buffer_write.joint_transforms[i] = joint_transforms[i].to_cols_array_2d();
        }
    }
}

pub struct FbxImportResult {
    pub static_meshes: Vec<StaticMesh>,
    pub animated_meshes: Vec<AnimatedMesh>
}

fn ufbx_vec3_to_glam(value: ufbx::Vec3) -> glam::Vec3 {
    glam::Vec3 {
        x: value.x as f32,
        y: value.y as f32,
        z: value.z as f32
    }
}

fn ufbx_vec2_to_glam(value: ufbx::Vec2) -> glam::Vec2 {
    glam::Vec2 {
        x: value.x as f32,
        y: value.y as f32,
    }
}

fn ufbx_matrix_to_glam(m: &ufbx::Matrix) -> Mat4 {
    Mat4::from_cols_array(&[
        m.m00 as f32, m.m10 as f32, m.m20 as f32, 0.0,
        m.m01 as f32, m.m11 as f32, m.m21 as f32, 0.0,
        m.m02 as f32, m.m12 as f32, m.m22 as f32, 0.0,
        m.m03 as f32, m.m13 as f32, m.m23 as f32, 1.0,
    ])
}

pub struct FbxImport;

impl FbxImport {
    pub fn from_bytes(bytes: &[u8], memory_allocator: Arc<StandardMemoryAllocator>) -> Result<FbxImportResult, Box<dyn std::error::Error>> {
        let mut static_meshes = Vec::new();
        let mut animated_meshes = Vec::new();

        let mut opts = LoadOpts::default();
        // This turns the model upside down because the up axis is negative Y in our engine
        opts.target_axes = ufbx::CoordinateAxes {
            right: ufbx::CoordinateAxis::PositiveX,
            up: ufbx::CoordinateAxis::NegativeY,
            front: ufbx::CoordinateAxis::PositiveZ
        };
        opts.space_conversion = ufbx::SpaceConversion::ModifyGeometry;

        let scene = ufbx::load_memory(bytes, opts)
            .map_err(|e| anyhow::anyhow!("FBX error: {e:?}"))
            .with_context(|| format!("Failed to load file"))?;

        let animations = Self::extract_animations(&scene);

        for mesh in &scene.meshes {
            let name: &str = &mesh.element.name;

            if mesh.materials.len() > 1 || mesh.material_parts.len() > 1 {
                println!("Skipping mesh '{name}': multi-material models are ignored.");
                continue;
            }

            println!("Mesh: {name}");

            match mesh.skin_deformers.is_empty() || animations.is_empty() {
                true => { // static mesh
                    let (vertices, indices) = Self::extract_static_vertices(mesh);

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

                    let static_mesh = StaticMesh {
                        index_buffer: Arc::new(index_buffer),
                        vertex_buffer: Arc::new(vertex_buffer)
                    };

                    static_meshes.push(static_mesh);
                },
                false => { // animated mesh
                    let (vertices, indices) = Self::extract_animated_vertices(mesh);

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

                    let (root_joint, joint_count) = Self::build_joint_tree(mesh).unwrap();

                    let animated_mesh = AnimatedMesh {
                        vertex_buffer: Arc::new(vertex_buffer),
                        index_buffer: Arc::new(index_buffer),
                        root_joint,
                        joint_count: joint_count as u32,
                        animator: Animator::new(animations.clone().into_iter().max_by(|a, b| a.length.cmp(&b.length)))
                    };

                    animated_meshes.push(animated_mesh);
                }
            }
        }

        Ok( FbxImportResult { static_meshes, animated_meshes })
    }

    fn extract_static_vertices(mesh: &ufbx::Mesh) -> (Vec<GameVertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(mesh.num_vertices);

        for i in 0..mesh.num_indices {
            let cp_id = mesh.vertex_indices[i] as usize;

            let position = ufbx_vec3_to_glam(mesh.vertices[cp_id]);

            let normal = if mesh.vertex_normal.exists {
                ufbx_vec3_to_glam(mesh.vertex_normal[i])
            } else {
                Vec3::ZERO
            };

            let uv = if mesh.vertex_uv.exists {
                ufbx_vec2_to_glam(mesh.vertex_uv[i])
            } else {
                Vec2::ZERO
            };

            let vertex = GameVertex {
                position: position.to_array(),
                normal: normal.to_array(),
                uv: uv.to_array()
            };

            vertices.push(vertex);
        }

        // triangulation
        let mut indices: Vec<u32> = Vec::new();
        let mut face_buffer: Vec<u32> = Vec::new();

        for face in &mesh.faces {
            face_buffer.clear();
            ufbx::triangulate_face_vec(&mut face_buffer, mesh, *face);
            indices.extend_from_slice(&face_buffer);
        }
        
        println!("fbx_import: imported one static mesh with {} vertices and {} indices", vertices.len(), indices.len());

        (vertices, indices)
    }

    fn extract_animated_vertices(mesh: &ufbx::Mesh) -> (Vec<AnimationVertex>, Vec<u32>) {
        #[derive(Default, Clone, Copy)]
        struct SkinWeight {
            pub joint_ids: [u32; 3],
            pub joint_weights: [f32; 3],
        }

        // getting the joints ids and weights for each bones
        let mut cp_weights = vec![SkinWeight::default(); mesh.num_vertices];
        if let Some(skin) = mesh.skin_deformers.first() {
            for (cluster_id, cluster) in skin.clusters.iter().enumerate() {
                let bone_id = cluster_id as u32;

                for (&cp_id, &weight) in cluster.vertices.iter().zip(cluster.weights.iter()) {
                    let cp_id = cp_id as usize;
                    if cp_id < cp_weights.len() {
                        let w = &mut cp_weights[cp_id];
                        for j in 0..3 {
                            if w.joint_weights[j] == 0.0 {
                                w.joint_ids[j] = bone_id;
                                w.joint_weights[j] = weight as f32;
                                break;
                            }
                        }
                    }
                }
            }
        }
        // normalize the weights of each bones to a sum of one
        for weight in cp_weights.iter_mut() {
            let sum: f32 = weight.joint_weights.iter().sum();
            if sum == 0.0 {
                continue;
            }
            let mul_factor = 1.0 / sum;
            weight
                .joint_weights
                .iter_mut()
                .for_each(|weight| *weight *= mul_factor);
        }

        let mut vertices = Vec::with_capacity(mesh.num_indices);

        for i in 0..mesh.num_indices {
            let cp_id = mesh.vertex_indices[i] as usize;
            let position = ufbx_vec3_to_glam(mesh.vertices[cp_id]);

            let normal = if mesh.vertex_normal.exists {
                ufbx_vec3_to_glam(mesh.vertex_normal[i])
            } else {
                Vec3::ZERO
            };

            let uv = if mesh.vertex_uv.exists {
                ufbx_vec2_to_glam(mesh.vertex_uv[i])
            } else {
                Vec2::ZERO
            };

            let skin = cp_weights[cp_id];

            let vertex = AnimationVertex {
                position: position.to_array(),
                normal: normal.to_array(),
                uv: uv.to_array(),
                joint_ids: skin.joint_ids,
                joint_weights: skin.joint_weights
            };

            vertices.push(vertex);
        }

        let indices: Vec<u32> = mesh.vertex_indices.iter().map(|&idx| idx).collect();

        println!("fbx_import: imported one animated mesh with {} vertices and {} indices", vertices.len(), indices.len());

        (vertices, indices)
    }

    fn build_joint_tree(mesh: &ufbx::Mesh) -> Option<(Joint, usize)> {
        let skin = mesh.skin_deformers.first()?;

        let root_cluster = skin.clusters.iter().find(|cluster| {
            let Some(bone_node) = &cluster.bone_node else { return false };

            match &bone_node.parent {
                None => true,
                Some(parent_node) => !skin.clusters.iter().any(|c| {
                    c.bone_node.as_ref().map_or(false, |n| {
                        n.element.element_id == parent_node.element.element_id
                    })
                }),
            }
        })?;

        let root_node = root_cluster.bone_node.as_ref()?;

        Some(Self::build_joint_recursive(root_node, skin))
    }

    // fn build_joint_recursive(node: &Node, skin: &SkinDeformer) -> (Joint, usize) {
    //     let mut n_joint = 0;

    //     let joint_id = skin
    //         .clusters
    //         .iter()
    //         .position(|c| {
    //             c.bone_node.as_ref().map_or(false, |n| {
    //                 n.element.element_id == node.element.element_id
    //             })
    //         })
    //         .unwrap_or(0) as u32;

    //     let inverse_bind_transform = skin
    //         .clusters
    //         .get(joint_id as usize)
    //         .map(|c| ufbx_matrix_to_glam(&c.geometry_to_bone))
    //         .unwrap_or(Mat4::IDENTITY);

    //     let local_bind_transform = ufbx_matrix_to_glam(&node.node_to_parent);

    //     let mut children = Vec::new();
    //     for child_node in &node.children {
    //         if skin.clusters.iter().any(|c| {
    //             c.bone_node.as_ref().map_or(false, |n| {
    //                 n.element.element_id == child_node.element.element_id
    //             })
    //         }) {
    //             let (joint, n) = Self::build_joint_recursive(child_node, skin);
    //             children.push(joint);
    //             n_joint += n + 1;
    //         }
    //     }

    //     (Joint {
    //         id: joint_id,
    //         name: node.element.name.to_string(),
    //         animated_transform: Mat4::IDENTITY,
    //         local_bind_transform,
    //         inverse_bind_transform,
    //         children
    //     }, n_joint)
    // }

    fn build_joint_recursive(node: &Node, skin: &SkinDeformer) -> (Joint, usize) {
        let mut joint_count = 0;

        let joint_id_opt = skin.clusters.iter().position(|c| {
            c.bone_node.as_ref().map_or(false, |n| n.element.element_id == node.element.element_id)
        });

        let inverse_bind_transform = {
            if let Some(id) = joint_id_opt {
                skin.clusters.get(id)
                    .map(|c| ufbx_matrix_to_glam(&c.geometry_to_bone))
                    .unwrap_or(Mat4::IDENTITY)
            } else {
                Mat4::IDENTITY
            }
        };

        let local_bind_transform = ufbx_matrix_to_glam(&node.node_to_parent);

        let mut children = Vec::new();
        for child_node in &node.children {
            if child_node.bone.is_some() {
                let (joint, n) = Self::build_joint_recursive(child_node, skin);
                children.push(joint);
                joint_count += n + 1;
            }
        }

        (Joint {
            id: joint_id_opt.unwrap_or(usize::MAX) as u32,
            name: node.element.name.to_string(),
            animated_transform: Mat4::IDENTITY,
            local_bind_transform,
            inverse_bind_transform,
            children
        }, joint_count)
    }

    fn extract_animations(scene: &SceneRoot) -> Vec<Animation> {
        let mut animations = Vec::new();
        let sample_rate = 30.0; // 30 FPS
        
        for anim_stack in &scene.anim_stacks {
            let duration_secs = anim_stack.time_end - anim_stack.time_begin;
            let total_frames = (duration_secs * sample_rate).ceil() as usize;

            let mut keyframes = Vec::with_capacity(total_frames);

            for frame_id in 0..total_frames {
                let time_sec = frame_id as f64 / sample_rate;
                let time_stamp = Duration::from_secs_f64(time_sec);

                let mut pose = HashMap::new();

                for node in &scene.nodes {
                    if node.bone.is_none() {
                        continue;
                    }

                    let bone_name = node.element.name.to_string();

                    let transform = ufbx::evaluate_transform(&anim_stack.anim, node, time_sec);

                    let position = Vec3 {
                        x: transform.translation.x as f32,
                        y: transform.translation.y as f32,
                        z: transform.translation.z as f32
                    };

                    let rotation = Quat::from_xyzw(
                        transform.rotation.x as f32,
                        transform.rotation.y as f32,
                        transform.rotation.z as f32,
                        transform.rotation.w as f32,
                    );

                    pose.insert(bone_name, JointTransform { position, rotation });
                }
                
                keyframes.push(KeyFrame { time_stamp, pose });
            }

            println!("fbx_import: imported one animation that lasts {} seconds", duration_secs);

            animations.push(Animation {
                length: Duration::from_secs_f64(duration_secs),
                frames: keyframes
            });

        }

        animations
    }
}
