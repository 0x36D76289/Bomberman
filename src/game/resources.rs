use std::{collections::HashMap, sync::Arc};

use include_dir::{include_dir, Dir, DirEntry};
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract},
    image::view::ImageView,
    memory::allocator::StandardMemoryAllocator,
};

use crate::graphics::{animation::{self, Animation}, mesh::FbxImport};
use crate::graphics::{StaticMesh, Vulkan, load_texture, mesh::Mesh, object::TextureIndex};

/// The list of objects a Game might require
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceName {
    Breakable,
    Unbreakable,
    Wall,
    Floor,
    Player,
    Bomb,
    PowerSpeed,
    PowerPower,
    PowerBomb,
    PowerSlide,
    FontAtlas,
    #[cfg(debug_assertions)]
    Test
}

/// the global [Resources] object saves the current textures and models in use
#[derive(Debug, Clone)]
pub struct Resources {
    pub textures: Vec<Arc<ImageView>>,
    pub textures_index: HashMap<String, TextureIndex>,
    pub models: HashMap<String, Vec<Mesh>>,
    pub animations: HashMap<String, Animation>
}

impl Resources {
    /// Executed at the start of the program it loads all the required data in memory
    pub fn load_resources(vulkan: &Vulkan) -> Self {
        // embed the texture and object files directory into the game executable
        static TEXTURES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/assets/textures");
        static MODELS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/assets/objects");
        static ANIMATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/assets/animations");

        let mut resources = Self {
            textures: Vec::new(),
            textures_index: HashMap::new(),
            models: HashMap::new(),
            animations: HashMap::new(),
        };

        // load the texture files
        resources.load_textures(&TEXTURES_DIR, vulkan);

        // load the object files
        resources.load_models(&MODELS_DIR, vulkan);
        resources.copy_model("quad", &["floor", "power_speed", "power_power", "power_bomb", "power_slide"]);
        resources.copy_model("cube", &["breakable", "unbreakable", "wall"]);

        // load the animation files
        resources.load_animations(&ANIMATIONS_DIR, vulkan);

        resources
    }

    // TODO: review doc
    /// Iterates over the files in src/assets/textures and fills the texture_array and texture_indexes vectors with the png files found
    /// The files are loaded into Vulkan memorey as ImageView that can be used directly by the graphics card
    fn load_textures(&mut self, textures_dir: &Dir, vulkan: &Vulkan) {
        let texture_array = &mut self.textures;
        let texture_indexes = &mut self.textures_index;

        let mut command_buffer = AutoCommandBufferBuilder::primary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        for entry in textures_dir.entries() {
            match entry {
                DirEntry::Dir(_) => {continue;}
                DirEntry::File(file) => {
                    if file.path().extension().is_some_and(|e| e == "png") {
                        let file_stem = file.path().file_stem().unwrap().to_str();
                        let file_stem = match file_stem {
                            None => {continue;}
                            Some(file_stem) => file_stem.to_string()
                        };
                        texture_array.push(
                            load_texture(file.contents(), &mut command_buffer, vulkan.memory_allocator.clone())
                        );
                        texture_indexes.insert(file_stem, (texture_array.len() - 1) as TextureIndex);
                    }
                }
            }
        }

        let _ = command_buffer
            .build()
            .unwrap()
            .execute(vulkan.queue.clone())
            .unwrap();
    }

    fn load_models(&mut self, models_dir: &Dir, vulkan: &Vulkan) {
        let model_map = &mut self.models;

        let memory_allocator = vulkan.memory_allocator.clone();

        for entry in models_dir.entries() {
            match entry {
                DirEntry::Dir(_) => {continue;}
                DirEntry::File(file) => {
                    if let Some(file_extension) = file.path().extension() {
                        let file_stem = file.path().file_stem().unwrap().to_str();
                        let file_stem = match file_stem {
                            None => {continue;}
                            Some(file_stem) => file_stem.to_string()
                        };
                        if file_extension == "obj" {
                            let obj_mesh = StaticMesh::load_from_obj(file.contents(), memory_allocator.clone()).unwrap();
                            model_map.insert(file_stem, vec![Mesh::Static(obj_mesh)]);
                        } else if file_extension == "fbx" {
                            let meshes = FbxImport::from_bytes(file.contents(), memory_allocator.clone()).unwrap();
                            if meshes.static_meshes.is_empty() && meshes.skinned_meshes.is_empty() {
                                panic!("Tried to import {} but found no mesh", file.path().to_str().unwrap())
                            }
                            let meshes = meshes.skinned_meshes.into_iter().map(|m| Mesh::Skinned(m))
                                .chain(meshes.static_meshes.into_iter().map(|m| Mesh::Static(m)))
                                .collect();
                            model_map.insert(file_stem, meshes);
                        }
                    }
                }
            }
        }
    }

    fn load_animations(&mut self, animations_dir: &Dir, vulkan: &Vulkan) {
        let animations_map = &mut self.animations;

        let memory_allocator = vulkan.memory_allocator.clone();

        for entry in animations_dir.entries() {
            match entry {
                DirEntry::Dir(_) => {continue;}
                DirEntry::File(file) => {
                    if let Some(file_extension) = file.path().extension() {
                        let file_stem = file.path().file_stem().unwrap().to_str();
                        let file_stem = match file_stem {
                            None => {continue;}
                            Some(file_stem) => file_stem.to_string()
                        };
                        if file_extension == "fbx" {
                            let meshes = FbxImport::from_bytes(file.contents(), memory_allocator.clone()).unwrap();
                            if meshes.animations.is_empty() {
                                panic!("Tried to import {} but found no animation", file.path().to_str().unwrap())
                            }
                            // using the longest anim found because a fbx file can contain multiple animations
                            let longest_anim = meshes.animations.clone().into_iter().max_by(|a, b| a.length.cmp(&b.length));
                            animations_map.insert(file_stem, longest_anim.unwrap());
                        }
                    }
                }
            }
        }
    }

    fn copy_model(&mut self, src: &str, dest: &[&str]) {
        if let Some(model) = self.models.get(src) {
            let model = model.clone();
            for d in dest {
                self.models.insert(d.to_string(), model.clone());
            }
        }
    }

    pub fn model(&self, name: &str) -> Vec<Mesh> {
        match self.models.get(name) {
            Some(model) => model.clone(),
            None => panic!("model '{name}' not found")
        }
    }

    pub fn texture(&self, name: &str) -> Option<TextureIndex> {
        if self.textures_index.contains_key(name) {
            Some(self.textures_index[name])
        } else {
            None
        }
    }

    pub fn animation(&self, name: &str) -> Animation {
        match self.animations.get(name) {
            Some(animation) => animation.clone(),
            None => panic!("animation '{name}' not found")
        }
    }
}
