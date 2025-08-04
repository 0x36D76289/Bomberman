use crate::game::state::State;
use crate::graphics::systems::point_light_render_system::{self, PointLightSystem};
use crate::graphics::systems::GlobalUbo;
use crate::graphics::systems::entity_render_system::{self, GameEntitySystem};
use crate::graphics::{
    self, load_texture, Camera, Entity, Graphics, Model, Physics, RenderContext, Renderer, Transform, Vulkan
};
use crate::input::{InputState, KeyboardMovementController};
use glam::{Vec3, Vec4};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract};
use vulkano::device::{Device, Queue};
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::StandardMemoryAllocator;
use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::KeyCode,
    window::WindowId,
};

pub struct App {
    pub state: State,
    pub graphics: Graphics
}

impl App {
    pub fn init(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let graphics = Graphics::new(event_loop)?;

        let (entities, textures) = load_entities(graphics.vulkan.memory_allocator.clone(), graphics.vulkan.command_buffer_allocator.clone(), graphics.vulkan.queue.clone())?;

        let state = State::default_state(entities, textures)?;

        Ok(Self {
            state,
            graphics
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let graphics = &mut self.graphics;

        graphics.renderer.init_render_context(event_loop, &graphics.vulkan);
        graphics.game_object_system.create_pipeline(
            &graphics.vulkan,
            graphics.renderer.get_rcx().render_pass.clone(),
            graphics.renderer.get_rcx().window.inner_size()
        );
        graphics.point_light_system.create_pipeline(
            &graphics.vulkan,
            graphics.renderer.get_rcx().render_pass.clone(),
            graphics.renderer.get_rcx().window.inner_size()
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // TODO: main loop
        // TODO: render
        
        // Event Handling
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.state.input_state.update_keyboard_input(event)
            }
            WindowEvent::RedrawRequested => {
                let state = &mut self.state;
                let renderer = &mut self.graphics.renderer;
                let game_object_system = &self.graphics.game_object_system;
                let point_light_system = &self.graphics.point_light_system;

                if (state.input_state.change_controller_target) {
                    state.controlled_object_id = (state.controlled_object_id + 1) % state.entities.len();
                    state.input_state.change_controller_target = false;
                }
                state.entity_controller.move_in_plane_xz(
                    &state.input_state,
                    renderer.get_delta_time(),
                    &mut state.entities[state.controlled_object_id],
                );
                state.camera.set_view_xyz(
                    state.entities[0].physics.unwrap().transform.translation,
                    state.entities[0].physics.unwrap().transform.rotation,
                );
                state.camera.set_perspective_projection(
                    0.872664626,
                    renderer.get_aspect_ration(),
                    0.1,
                    100.0,
                );
    
                if state.input_state.debug {
                    state.input_state.debug = false;
                    state.debug();
                }

                if let Some(mut command_buffer) = renderer.begin_frame(&self.graphics.vulkan) {
                    let (lights, light_number) = point_light_system.lights_array(&state.entities);
                    let mut global_ubo = GlobalUbo {
                        projection: state.camera.projection_matrix.to_cols_array_2d(),
                        view: state.camera.view_matrix.to_cols_array_2d(),
                        inverse_view: state.camera.inverse_view_matrix.to_cols_array_2d(),
                        ambient_light_color: [1.0, 1.0, 1.0, 0.2],
                        lights: lights,
                        light_number: light_number
                    };

                    game_object_system.render_game_objects(&self.graphics.vulkan, &state.entities, &state.textures, global_ubo, &mut command_buffer);
                    point_light_system.render(&self.graphics.vulkan, &state.entities, global_ubo, &mut command_buffer);
                    // point_light_system.update(&mut state.light, renderer.get_delta_time());
                    renderer.end_frame(&self.graphics.vulkan, command_buffer);
                    renderer.update_time();
                    renderer.update_title(
                        &format!("Bomberman!! fps: {:.0} control: {}", renderer.get_rcx().time_info.avg_fps, state.entities[state.controlled_object_id].name.as_ref().unwrap_or(&"undefined".to_string()))
                    );
                }
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
        CommandBufferUsage::OneTimeSubmit
    )
    .unwrap();

    let mut textures = Vec::new();
    textures.push(load_texture(include_bytes!("assets/WhiteBomberMan.png"), &mut command_buffer, memory_allocator.clone()));
    textures.push(load_texture(include_bytes!("assets/crate2.png"), &mut command_buffer, memory_allocator.clone()));
    textures.push(load_texture(include_bytes!("assets/textureStone.png"), &mut command_buffer, memory_allocator.clone()));
    let _ = command_buffer.build().unwrap().execute(queue.clone()).unwrap();

    // load the models
    let bomberman_model = Model::load(include_bytes!("assets/bomberman.obj"), memory_allocator.clone())?;
    let cube_model = Model::load(include_bytes!("assets/cube.obj"), memory_allocator.clone())?;
    let quad_model = Model::load(include_bytes!("assets/quad.obj"), memory_allocator.clone())?;
    
    // create the entities
    let mut entities = Vec::new();

    entities.push(Entity::default()
        .with_name("Camera")
        .with_position(Vec3::new(0.0, -5.25, -3.0))
        .with_rotation(Vec3::new(-1.17, 0.0, 0.0))
    );

    entities.push(Entity::default()
        .with_name("Bomberman")
        .with_model(bomberman_model.clone())
        .with_texture(0)
        .with_position(Vec3::new(-0.5, 0.5, 0.0))
        .with_scale(Vec3::splat(0.15))
    );

    entities.push(Entity::default()
        .with_name("Crate1")
        .with_model(cube_model.clone())
        .with_texture(1)
        .with_position(Vec3::new(0.5, 0.5, 0.0))
        .with_scale(Vec3::splat(0.5))
    );

    entities.push(Entity::default()
        .with_name("Floor")
        .with_model(quad_model.clone())
        .with_color(Vec3::new(1.0, 1.0, 1.0))
        .with_position(Vec3::new(0.0, 0.5, 0.0))
        .with_scale(Vec3::new(3.0, 1.0, 3.0))
    );

    entities.push(Entity::default()
        .with_name("Light1")
        .with_color(Vec3::ONE)
        .with_light(1.0)
        .with_position(Vec3::new(-1.0, -0.75, 0.0))
        .with_scale(Vec3::splat(0.1))
    );

    entities.push(Entity::default()
        .with_name("Light2")
        .with_color(Vec3::new(1.0, 0.0, 0.0))
        .with_light(1.0)
        .with_position(Vec3::new(1.0, -0.75, 0.0))
        .with_scale(Vec3::splat(0.1))
    );

    Ok((entities, textures))
}
