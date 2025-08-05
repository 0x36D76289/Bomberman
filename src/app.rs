use crate::game::Entity;
use crate::game::state::State;
use crate::graphics::{GlobalUbo, Graphics, Model, load_texture};
use glam::Vec3;
use rand::Rng;
use core::f32;
use std::error::Error;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract,
};
use vulkano::device::Queue;
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::StandardMemoryAllocator;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

pub struct App {
    pub state: State,
    pub graphics: Graphics,
}

impl App {
    pub fn init(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let graphics = Graphics::new(event_loop)?;

        let (entities, textures) = load_entities(
            graphics.vulkan.memory_allocator.clone(),
            graphics.vulkan.command_buffer_allocator.clone(),
            graphics.vulkan.queue.clone(),
        )?;

        let state = State::default_state(entities, textures)?;

        Ok(Self { state, graphics })
    }

    fn update_world(&mut self) {
        let state = &mut self.state;

        if state.input_state.next_entity {
            state.controlled_object_id =
                (state.controlled_object_id + 1) % state.entities.len();
            state.input_state.next_entity = false;
        }
        if state.input_state.previous_entity {
            state.controlled_object_id =
                (state.controlled_object_id - 1) % state.entities.len();
            state.input_state.previous_entity = false;
        }

        if let Some(name) = &state.entities[state.controlled_object_id].name && name == "Player" {
            state.entity_controller.move_in_plane_xz_player(
                &state.input_state,
                self.graphics.renderer.get_delta_time(),
                &mut state.entities[state.controlled_object_id],
            );
        } else {
            state.entity_controller.move_in_plane_xz(
                &state.input_state,
                self.graphics.renderer.get_delta_time(),
                &mut state.entities[state.controlled_object_id],
            );
        }

        state.camera.set_view_xyz(
            state.entities[0].physics.unwrap().transform.translation,
            state.entities[0].physics.unwrap().transform.rotation,
        );
        state.camera.set_perspective_projection(
            0.6,
            self.graphics.renderer.get_aspect_ration(),
            0.1,
            100.0,
        );

        if state.input_state.debug {
            state.input_state.debug = false;
            state.debug();
        }
    }

    fn draw_frame(&mut self) {
        let state = &mut self.state;
        let renderer = &mut self.graphics.renderer;
        let game_object_system = &self.graphics.game_object_system;
        let point_light_system = &self.graphics.point_light_system;

        if let Some(mut command_buffer) = renderer.begin_frame(&self.graphics.vulkan) {
            let (lights, light_number) = point_light_system.lights_array(&state.entities);
            let global_ubo = GlobalUbo {
                projection: state.camera.projection_matrix.to_cols_array_2d(),
                view: state.camera.view_matrix.to_cols_array_2d(),
                inverse_view: state.camera.inverse_view_matrix.to_cols_array_2d(),
                ambient_light_color: [1.0, 1.0, 1.0, 0.8],
                lights: lights,
                light_number: light_number,
            };

            game_object_system.render_game_objects(
                &self.graphics.vulkan,
                &state.entities,
                &state.textures,
                global_ubo,
                &mut command_buffer,
            );
            point_light_system.render(
                &self.graphics.vulkan,
                &state.entities,
                global_ubo,
                &mut command_buffer,
            );
            // point_light_system.update(&mut state.light, renderer.get_delta_time());
            renderer.end_frame(&self.graphics.vulkan, command_buffer);
            renderer.update_time();
            renderer.update_title(&format!(
                "Bomberman!! fps: {:.0} control: {} {}",
                renderer.get_rcx().time_info.avg_fps,
                state.entities[state.controlled_object_id]
                    .name
                    .as_ref()
                    .unwrap_or(&"undefined".to_string()),
                state.entities[state.controlled_object_id]
                    .physics
                    .as_ref()
                    .unwrap()
                    .transform
                    .translation
            ));
        }
    }
}

impl ApplicationHandler for App {
    // This is called when the window is created
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let graphics = &mut self.graphics;

        graphics
            .renderer
            .init_render_context(event_loop, &graphics.vulkan);
        graphics.game_object_system.create_pipeline(
            &graphics.vulkan,
            graphics.renderer.get_rcx().render_pass.clone(),
        );
        graphics.point_light_system.create_pipeline(
            &graphics.vulkan,
            graphics.renderer.get_rcx().render_pass.clone(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // TODO: main loop
        // TODO: render

        // Event Handling
        match event {
            // Handle inputs
            WindowEvent::KeyboardInput { event, .. } => {
                self.state.input_state.update_keyboard_input(event)
            }
            WindowEvent::RedrawRequested => {
                self.update_world();
                self.draw_frame();
            }
            WindowEvent::Resized(_) => self.graphics.renderer.recreate_swapchain(true),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.renderer.request_redraw();
    }
}

fn load_entities(
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
) -> Result<(Vec<Entity>, Vec<Arc<ImageView>>), Box<dyn Error>> {
    // load the textures
    let mut command_buffer = AutoCommandBufferBuilder::primary(
        command_buffer_allocator.clone(),
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let mut textures = Vec::new();
    textures.push(load_texture(
        include_bytes!("assets/WhiteBomberMan.png"),
        &mut command_buffer,
        memory_allocator.clone(),
    ));
    textures.push(load_texture(
        include_bytes!("assets/000-floor.png"),
        &mut command_buffer,
        memory_allocator.clone(),
    ));
    textures.push(load_texture(
        include_bytes!("assets/001-durable_wall.png"),
        &mut command_buffer,
        memory_allocator.clone(),
    ));
    textures.push(load_texture(
        include_bytes!("assets/025-noteblock.png"),
        &mut command_buffer,
        memory_allocator.clone(),
    ));
    let _ = command_buffer
        .build()
        .unwrap()
        .execute(queue.clone())
        .unwrap();

    // load the models
    let bomberman_model = Model::load(
        include_bytes!("assets/bomberman.obj"),
        memory_allocator.clone(),
    )?;
    // let cube_model = Model::load(include_bytes!("assets/cube.obj"), memory_allocator.clone())?;
    // let quad_model = Model::load(include_bytes!("assets/quad.obj"), memory_allocator.clone())?;

    // create the entities
    let mut entities = Vec::new();

    entities.push(
        Entity::default()
            .with_name("Camera")
            .with_position(Vec3::new(0.0, -19.0, -9.0))
            .with_rotation(Vec3::new(-1.17, 0.0, 0.0))
    );
    
    entities.push(
        Entity::default()
            .with_name("Player")
            .with_model(bomberman_model.clone())
            .with_texture(0)
            .with_position(Vec3::new(0.0, 0.5, 0.0))
            .with_scale(Vec3::new(0.3, 0.3, 0.3))
    );

    entities.push(
        Entity::default()
            .with_name("Light")
            .with_color(Vec3::ONE)
            .with_light(40.0)
            .with_position(Vec3::new(0.0, -7.0, 2.0))
            .with_scale(Vec3::splat(0.1))
    );

    create_map(&mut entities, &memory_allocator).unwrap();

    Ok((entities, textures))
}

const MAP_WIDTH: usize = 13;
const MAP_HEIGTH: usize = 11;

fn create_map(entities: &mut Vec<Entity>, memory_allocator: &Arc<StandardMemoryAllocator>) -> Result<(), Box<dyn Error>> {
    let cube_model = Model::load(include_bytes!("assets/cube.obj"), memory_allocator.clone())?;
    let quad_model = Model::load(include_bytes!("assets/quad.obj"), memory_allocator.clone())?;

    // create floor
    let floor_base = Entity::default()
        .with_model(quad_model.clone())
        .with_texture(1)
        .with_scale(Vec3::ONE);
    for row in 1..(MAP_HEIGTH - 1) {
        for col in 1..(MAP_WIDTH - 1) {
            let x: f32 = -(MAP_WIDTH as f32 / 2.0) + 0.5 + col as f32;
            let z: f32 = -(MAP_HEIGTH as f32 / 2.0) + 0.5 + row as f32;
            entities.push(
                floor_base
                    .clone()
                    .with_name(&format!("floor_{row}_{col}"))
                    .with_position(Vec3::new(x, 0.5, z))
            );
        }
    }

    // create wall blocks
    let wall_block = Entity::default()
        .with_model(cube_model.clone())
        .with_texture(2)
        .with_scale(Vec3::splat(0.9));
    // for row in 1..(MAP_HEIGTH - 1) / 2 {
    //     for col in 1..(MAP_WIDTH - 1) / 2 {
    //         let x: f32 = -(MAP_WIDTH as f32 / 2.0) + 0.5 + 2.0 * col as f32;
    //         let z: f32 = -(MAP_HEIGTH as f32 / 2.0) + 0.5 + 2.0 * row as f32;
    //         entities.push(
    //             wall_block
    //                 .clone()
    //                 .with_name(&format!("wall_block_{row}_{col}"))
    //                 .with_position(Vec3::new(x, 0.5, z))
    //         );
    //     }
    // }

    // create crates
    let crate_block = Entity::default()
        .with_model(cube_model.clone())
        .with_texture(3)
        .with_scale(Vec3::splat(0.9));
    let mut rng = rand::rng();
    for row in 1..(MAP_HEIGTH - 1) {
        for col in 1..(MAP_WIDTH - 1) {
            let x: f32 = -(MAP_WIDTH as f32 / 2.0) + 0.5 + col as f32;
            let z: f32 = -(MAP_HEIGTH as f32 / 2.0) + 0.5 + row as f32;
            if row % 2 == 0 && col % 2 == 0 {
                entities.push(
                    wall_block
                        .clone()
                        .with_name(&format!("wall_block_{row}_{col}"))
                        .with_position(Vec3::new(x, 0.5, z))
                );
            } else {
                if rng.random_range(0..=1) == 1 {continue;}
                entities.push(
                    crate_block
                        .clone()
                        .with_name(&format!("crate_block_{row}_{col}"))
                        .with_position(Vec3::new(x, 0.5, z))
                        .with_rotation(Vec3::new(0.0, rng.random_range(-0.05..0.05), 0.0))
                );
            }
        }
    }

    Ok(())
}