use crate::game::state::State;
use crate::graphics::systems::point_light_system::{self, PointLightSystem};
use crate::graphics::systems::GlobalUbo;
use crate::graphics::systems::game_object_system::{self, GameEntitySystem};
use crate::graphics::{
    self, load_texture, Camera, GameEntity, GameEntityType, Graphics, Model, RenderContext, Renderer, Transform, Vulkan
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

        let (objects, textures) = load_game_objects(graphics.vulkan.memory_allocator.clone(), graphics.vulkan.command_buffer_allocator.clone(), graphics.vulkan.queue.clone())?;

        let state = State::default_state(objects, textures)?;

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
                    state.entities[0].transform.translation,
                    state.entities[0].transform.rotation,
                );
                state.camera.set_perspective_projection(
                    0.872664626,
                    renderer.get_aspect_ration(),
                    0.1,
                    100.0,
                );

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
                        &format!("Bomberman!! fps: {:.0} control: {}", renderer.get_rcx().time_info.avg_fps, state.entities[state.controlled_object_id].name)
                    );
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.state.input_state.update_keyboard_input(event)
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

fn load_game_objects(
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: Arc<Queue>,
) -> Result<(Vec<GameEntity>, Vec<Arc<ImageView>>), Box<dyn Error>> {
    let mut command_buffer = AutoCommandBufferBuilder::primary(
        command_buffer_allocator.clone(),
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit
    )
    .unwrap();

    let mut textures = Vec::new();
    textures.push(load_texture(include_bytes!("assets/WhiteBomberMan.png"), &mut command_buffer, memory_allocator.clone()));
    textures.push(load_texture(&include_bytes!("assets/crate2.png").to_vec(), &mut command_buffer, memory_allocator.clone()));
    textures.push(load_texture(&include_bytes!("assets/textureStone.png").to_vec(), &mut command_buffer, memory_allocator.clone()));
    let _ = command_buffer.build().unwrap().execute(queue.clone()).unwrap();

    let mut viewer_object = GameEntity::new(GameEntityType::Viewer);
    viewer_object.name = "Camera".to_string();
    viewer_object.transform.translation = Vec3::new(0.0, -5.25, -3.0);
    viewer_object.transform.rotation.x = -1.17;

    let model = Model::load(include_bytes!("assets/bomberman.obj"), memory_allocator.clone())?;
    let mut bomberman = GameEntity::new_object("Bomberman", model.clone(), Some(0), Vec3::new(1.0, 1.0, 1.0));
    bomberman.transform.translation = Vec3::new(-0.5, 0.5, 0.0);
    bomberman.transform.scale = Vec3::splat(0.1);

    let model = Model::load(include_bytes!("assets/cube.obj"), memory_allocator.clone())?;
    let mut cratee = GameEntity::new_object("Crate", model.clone(), Some(1), Vec3::new(1.0, 1.0, 1.0));
    cratee.transform.translation = Vec3::new(0.5, 0.5, 0.0);
    cratee.transform.scale = Vec3::splat(0.5);

    let model = Model::load(include_bytes!("assets/quad.obj"), memory_allocator.clone())?;
    let mut floor = GameEntity::new_object("Floor", model.clone(), None, Vec3::new(1.0, 1.0, 1.0));
    floor.transform.translation = Vec3::new(0.0, 0.5, 0.0);
    floor.transform.scale = Vec3::new(3.0, 1.0, 3.0);

    let mut light1 = GameEntity::new(GameEntityType::Light { color: Vec4::splat(1.0) });
    light1.name = "Light1".to_string();
    light1.transform.translation = Vec3::new(-1.5, -0.75, -1.5);
    light1.transform.scale = Vec3::splat(0.1);

    let mut light2 = GameEntity::new(GameEntityType::Light { color: Vec4::new(1.0, 0.0, 0.0, 1.0) });
    light2.name = "Light2".to_string();
    light2.transform.translation = Vec3::new(-1.5, -0.75, 1.5);
    light2.transform.scale = Vec3::splat(0.1);

    let mut light3 = GameEntity::new(GameEntityType::Light { color: Vec4::new(0.0, 1.0, 0.0, 1.0) });
    light3.name = "Light3".to_string();
    light3.transform.translation = Vec3::new(1.5, -0.75, -1.5);
    light3.transform.scale = Vec3::splat(0.1);

    let mut light4 = GameEntity::new(GameEntityType::Light { color: Vec4::new(0.0, 0.0, 1.0, 1.0) });
    light4.name = "Light4".to_string();
    light4.transform.translation = Vec3::new(1.5, -0.75, 1.5);
    light4.transform.scale = Vec3::splat(0.1);

    let objects = vec![viewer_object, bomberman, floor, cratee, light1, light2, light3, light4];

    Ok((objects, textures))
}
