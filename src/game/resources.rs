use std::{collections::HashMap, sync::Arc};

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract},
    image::view::ImageView,
    memory::allocator::StandardMemoryAllocator,
};

use crate::graphics::mesh::FbxImport;
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
    pub textures_index: HashMap<ResourceName, TextureIndex>,
    pub models: HashMap<ResourceName, Mesh>,
}


enum ModelFileType {
    Obj,
    Fbx
}

struct ModelFile {
    file_type: ModelFileType,
    file_bytes: &'static [u8]
}

impl ModelFile {
    fn new(file_type: ModelFileType, file_bytes: &'static [u8]) -> Self {
        Self {
            file_type,
            file_bytes
        }
    }
}

impl Resources {
    /// Executed at the start of the program it loads all the required data in memory
    pub fn load_resources(vulkan: &Vulkan) -> Self {
        let mut textures: HashMap<ResourceName, &[u8]> = HashMap::new();
        let mut models: HashMap<ResourceName, ModelFile> = HashMap::new();

        // load the textures
        textures.insert(
            ResourceName::Player,
            include_bytes!("../assets/WhiteBomberMan.png"),
        );
        textures.insert(
            ResourceName::Breakable,
            include_bytes!("../assets/025-noteblock.png"),
        );
        textures.insert(
            ResourceName::Unbreakable,
            include_bytes!("../assets/001-durable_wall.png"),
        );
        textures.insert(
            ResourceName::Wall,
            include_bytes!("../assets/001-durable_wall.png"),
        );
        textures.insert(
            ResourceName::Floor,
            include_bytes!("../assets/000-floor.png"),
        );
        textures.insert(ResourceName::Bomb, include_bytes!("../assets/miku.png"));
        textures.insert(
            ResourceName::PowerSpeed,
            include_bytes!("../assets/simple.png"),
        );
        textures.insert(
            ResourceName::PowerPower,
            include_bytes!("../assets/WhiteBomberMan.png"),
        );
        textures.insert(
            ResourceName::PowerBomb,
            include_bytes!("../assets/textureStone.png"),
        );
        textures.insert(
            ResourceName::PowerSlide,
            include_bytes!("../assets/denji.png"),
        );
        textures.insert(
            ResourceName::FontAtlas,
            include_bytes!("../assets/font_atlas.png"),
        );
        #[cfg(debug_assertions)]
        textures.insert(
            ResourceName::Test,
            include_bytes!("../assets/Valkyrie.png"),
        );

        // load the models
        models.insert(
            ResourceName::Player,
            ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/bomberman.obj"))
        );
        models.insert(
            ResourceName::Breakable,
            ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/cube.obj"))
        );
        models.insert(
            ResourceName::Unbreakable,
            ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/cube.obj"))
        );
        models.insert(ResourceName::Wall, ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/cube.obj")));
        models.insert(ResourceName::Floor, ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/quad.obj")));
        models.insert(ResourceName::Bomb, ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/bomb.obj")));
        models.insert(
            ResourceName::PowerSpeed,
            ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/quad.obj"))
        );
        models.insert(
            ResourceName::PowerPower,
            ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/quad.obj"))
        );
        models.insert(
            ResourceName::PowerBomb,
            ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/quad.obj"))
        );
        models.insert(
            ResourceName::PowerSlide,
            ModelFile::new(ModelFileType::Obj, include_bytes!("../assets/quad.obj"))
        );
        #[cfg(debug_assertions)]
        models.insert(
            ResourceName::Test,
            ModelFile::new(ModelFileType::Fbx, include_bytes!("../assets/animation_test.fbx"))
        );

        let (textures, textures_index) = Resources::load_textures(textures, vulkan);
        let models = Resources::load_models(models, vulkan.memory_allocator.clone());

        Self {
            textures,
            textures_index,
            models,
        }
    }

    // TODO: review doc
    /// Loads the textures into the Vulkan memory, making them able to be rendered
    fn load_textures(
        textures: HashMap<ResourceName, &[u8]>,
        vulkan: &Vulkan,
    ) -> (Vec<Arc<ImageView>>, HashMap<ResourceName, TextureIndex>) {
        let mut texture_array: Vec<Arc<ImageView>> = Vec::new();
        let mut texture_indexes: HashMap<ResourceName, TextureIndex> = HashMap::new();

        let mut command_buffer = AutoCommandBufferBuilder::primary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        for texture in textures {
            texture_array.push(load_texture(
                texture.1,
                &mut command_buffer,
                vulkan.memory_allocator.clone(),
            ));
            texture_indexes.insert(texture.0, (texture_array.len() - 1) as TextureIndex);
        }

        let _ = command_buffer
            .build()
            .unwrap()
            .execute(vulkan.queue.clone())
            .unwrap();

        (texture_array, texture_indexes)
    }

    // TODO: review doc
    /// Loads the model into the Vulkan memory, making them able to be rendered
    fn load_models(
        models: HashMap<ResourceName, ModelFile>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> HashMap<ResourceName, Mesh> {
        let mut model_map: HashMap<ResourceName, Mesh> = HashMap::new();

        for model in models {
            let mesh = match model.1.file_type {
                ModelFileType::Obj => {
                    let obj_mesh = StaticMesh::load_from_obj(model.1.file_bytes, memory_allocator.clone()).unwrap();
                    Mesh::Static(obj_mesh)
                }
                // naive extraction: we take the first model we find and insert it into the model map
                ModelFileType::Fbx => {
                    let mut meshes = FbxImport::from_bytes(model.1.file_bytes, memory_allocator.clone()).unwrap();
                    match (!meshes.animated_meshes.is_empty(), !meshes.static_meshes.is_empty()) {
                        (true, _) => Mesh::Animated(meshes.animated_meshes.remove(0)),
                        (false, true) => Mesh::Static(meshes.static_meshes.remove(0)),
                        (false, false) => panic!("Tried to import an FBX file but found no mesh")
                    }
                }
            };

            model_map.insert(model.0, mesh);
        }

        model_map
    }
}
