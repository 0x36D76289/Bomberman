use std::{collections::HashMap, sync::Arc};

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract},
    image::view::ImageView,
    memory::allocator::StandardMemoryAllocator,
};

use crate::graphics::{Model, Vulkan, load_texture, object::TextureIndex};

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
}

/// the global [Resources] object saves the current textures and models in use
#[derive(Debug, Clone)]
pub struct Resources {
    pub textures: Vec<Arc<ImageView>>,
    pub textures_index: HashMap<ResourceName, TextureIndex>,
    pub models: HashMap<ResourceName, Arc<Model>>,
}

impl Resources {
    /// Executed at the start of the program it loads all the required data in memory
    pub fn load_resources(vulkan: &Vulkan) -> Self {
        let mut textures: HashMap<ResourceName, &[u8]> = HashMap::new();
        let mut models: HashMap<ResourceName, &[u8]> = HashMap::new();

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

        // load the models
        models.insert(
            ResourceName::Player,
            include_bytes!("../assets/bomberman.obj"),
        );
        models.insert(
            ResourceName::Breakable,
            include_bytes!("../assets/cube.obj"),
        );
        models.insert(
            ResourceName::Unbreakable,
            include_bytes!("../assets/cube.obj"),
        );
        models.insert(ResourceName::Wall, include_bytes!("../assets/cube.obj"));
        models.insert(ResourceName::Floor, include_bytes!("../assets/quad.obj"));
        models.insert(ResourceName::Bomb, include_bytes!("../assets/bomb.obj"));
        models.insert(
            ResourceName::PowerSpeed,
            include_bytes!("../assets/quad.obj"),
        );
        models.insert(
            ResourceName::PowerPower,
            include_bytes!("../assets/quad.obj"),
        );
        models.insert(
            ResourceName::PowerBomb,
            include_bytes!("../assets/quad.obj"),
        );
        models.insert(
            ResourceName::PowerSlide,
            include_bytes!("../assets/quad.obj"),
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
        models: HashMap<ResourceName, &[u8]>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> HashMap<ResourceName, Arc<Model>> {
        let mut model_map: HashMap<ResourceName, Arc<Model>> = HashMap::new();

        for model in models {
            model_map.insert(
                model.0,
                Model::load(model.1, memory_allocator.clone()).unwrap(),
            );
        }

        model_map
    }
}
