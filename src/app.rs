use crate::game::state::State;
use crate::graphics::systems::point_light_system::{self, PointLightSystem};
use crate::graphics::systems::GlobalUbo;
use crate::graphics::systems::game_object_system::{self, GameObjectSystem};
use crate::graphics::{
    self, load_texture, Camera, GameObject, Graphics, Light, Model, RenderContext, Renderer, Transform, Vulkan
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
                    state.controlled_object_id = (state.controlled_object_id + 1) % state.objects.len();
                    state.input_state.change_controller_target = false;
                }
                state.camera_controller.move_in_plane_xz(
                    &state.input_state,
                    renderer.get_delta_time(),
                    &mut state.objects[state.controlled_object_id],
                );
                state.camera.set_view_xyz(
                    state.objects[0].transform.translation,
                    state.objects[0].transform.rotation,
                );
                state.camera.set_perspective_projection(
                    0.872664626,
                    renderer.get_aspect_ration(),
                    0.1,
                    100.0,
                );

                if let Some(mut command_buffer) = renderer.begin_frame(&self.graphics.vulkan) {
                    let global_ubo = GlobalUbo {
                        projection: state.camera.projection_matrix.to_cols_array_2d(),
                        view: state.camera.view_matrix.to_cols_array_2d(),
                        inverse_view: state.camera.inverse_view_matrix.to_cols_array_2d(),
                        ambient_light_color: [1.0, 1.0, 1.0, 0.2],
                        light_position: state.light.transform.translation.to_array().into(),
                        light_color: state.light.color.to_array(),
                    };

                    game_object_system.render_game_objects(&self.graphics.vulkan, &state.objects, &state.textures, global_ubo, &mut command_buffer);
                    point_light_system.render(&self.graphics.vulkan, global_ubo, &mut command_buffer);
                    point_light_system.update(&mut state.light, renderer.get_delta_time());
                    renderer.end_frame(&self.graphics.vulkan, command_buffer);
                    renderer.update_time();
                    renderer.update_title();
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
) -> Result<(Vec<GameObject>, Vec<Arc<ImageView>>), Box<dyn Error>> {
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

    let mut viewer_object = GameObject::new();
    viewer_object.transform.translation = Vec3::new(0.0, -5.25, -3.0);
    viewer_object.transform.rotation.x = -1.17;

    let model = Model::load(include_bytes!("assets/bomberman.obj"), memory_allocator.clone())?;
    let mut bomberman = GameObject::new();
    bomberman.model = Some(model.clone());
    bomberman.texture_index = Some(0);
    bomberman.transform.translation = Vec3::new(-0.5, 0.5, 0.0);
    bomberman.transform.scale = Vec3::splat(0.1);
    bomberman.color = Vec3::new(1.0, 1.0, 1.0);

    let model = Model::load(include_bytes!("assets/cube.obj"), memory_allocator.clone())?;
    let mut cratee = GameObject::new();
    cratee.model = Some(model.clone());
    cratee.texture_index = Some(1);
    cratee.transform.translation = Vec3::new(0.5, 0.5, 0.0);
    cratee.transform.scale = Vec3::splat(0.5);
    cratee.color = Vec3::new(1.0, 1.0, 1.0);

    let model = Model::load(include_bytes!("assets/quad.obj"), memory_allocator.clone())?;
    let mut floor = GameObject::new();
    floor.model = Some(model.clone());
    floor.transform.translation = Vec3::new(0.0, 0.5, 0.0);
    floor.transform.scale = Vec3::new(3.0, 1.0, 3.0);

    let objects = vec![viewer_object, bomberman, floor, cratee];

    Ok((objects, textures))
}
