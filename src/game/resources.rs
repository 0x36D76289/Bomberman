use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract},
    image::view::ImageView,
};

use crate::graphics::{Model, Vulkan, load_texture};

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
}

const RESOURCE_NAME_SIZE: usize = ResourceName::PowerSlide as usize + 1;

#[derive(Debug, Clone)]
pub struct Resources {
    pub textures: [Arc<ImageView>; RESOURCE_NAME_SIZE],
    pub models: [Arc<Model>; RESOURCE_NAME_SIZE],
}

impl Resources {
    pub fn load_resources(vulkan: &Vulkan) -> Self {
        let memory_allocator = vulkan.memory_allocator.clone();

        let mut textures = vec![None; RESOURCE_NAME_SIZE];
        let mut models = vec![None; RESOURCE_NAME_SIZE];

        // load the textures
        let mut command_buffer = AutoCommandBufferBuilder::primary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        textures[ResourceName::Player as usize] = Some(load_texture(
            include_bytes!("../assets/WhiteBomberMan.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::Breakable as usize] = Some(load_texture(
            include_bytes!("../assets/025-noteblock.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::Unbreakable as usize] = Some(load_texture(
            include_bytes!("../assets/001-durable_wall.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::Wall as usize] = Some(load_texture(
            include_bytes!("../assets/001-durable_wall.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::Floor as usize] = Some(load_texture(
            include_bytes!("../assets/000-floor.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::Bomb as usize] = Some(load_texture(
            include_bytes!("../assets/miku.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::PowerSpeed as usize] = Some(load_texture(
            include_bytes!("../assets/simple.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::PowerPower as usize] = Some(load_texture(
            include_bytes!("../assets/WhiteBomberMan.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::PowerBomb as usize] = Some(load_texture(
            include_bytes!("../assets/textureStone.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        textures[ResourceName::PowerSlide as usize] = Some(load_texture(
            include_bytes!("../assets/denji.png"),
            &mut command_buffer,
            memory_allocator.clone(),
        ));
        let _ = command_buffer
            .build()
            .unwrap()
            .execute(vulkan.queue.clone())
            .unwrap();

        // load the models
        models[ResourceName::Player as usize] = Some(
            Model::load(
                include_bytes!("../assets/bomberman.obj"),
                memory_allocator.clone(),
            )
            .unwrap(),
        );
        models[ResourceName::Breakable as usize] = Some(
            Model::load(
                include_bytes!("../assets/cube.obj"),
                memory_allocator.clone(),
            )
            .unwrap(),
        );
        models[ResourceName::Unbreakable as usize] = Some(
            Model::load(
                include_bytes!("../assets/cube.obj"),
                memory_allocator.clone(),
            )
            .unwrap(),
        );
        models[ResourceName::Wall as usize] = Some(
            Model::load(
                include_bytes!("../assets/cube.obj"),
                memory_allocator.clone(),
            )
            .unwrap(),
        );
        models[ResourceName::Floor as usize] = Some(
            Model::load(
                include_bytes!("../assets/quad.obj"),
                memory_allocator.clone(),
            )
            .unwrap(),
        );
        models[ResourceName::Bomb as usize] = Some(
            Model::load(
                include_bytes!("../assets/bomb.obj"),
                memory_allocator.clone(),
            )
            .unwrap(),
        );

        let power_model = Model::load(
            include_bytes!("../assets/quad.obj"),
            memory_allocator.clone(),
        )
        .unwrap();
        models[ResourceName::PowerSpeed as usize] = Some(power_model.clone());
        models[ResourceName::PowerPower as usize] = Some(power_model.clone());
        models[ResourceName::PowerBomb as usize] = Some(power_model.clone());
        models[ResourceName::PowerSlide as usize] = Some(power_model.clone());

        // println!("{:?}", textures);

        let models: Vec<Arc<Model>> = models.into_iter().flatten().collect();
        let textures: Vec<Arc<ImageView>> = textures.into_iter().flatten().collect();

        Self {
            models: models.try_into().unwrap(),
            textures: textures.try_into().unwrap(),
        }
    }
}
