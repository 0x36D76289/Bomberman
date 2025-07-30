use crate::game::state::State;
use crate::graphics::{Camera, GameObject, Model, RenderContext, Transform, Vulkan};
use crate::input::{InputState, KeyboardMovementController};
use crate::load_model;
use glam::{Vec3, Vec4};
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
    pub input_state: InputState,
    pub objects: Vec<GameObject>,
    pub camera: Camera,
    pub viewer_object: GameObject,
    pub camera_controller: KeyboardMovementController,
    pub vulkan: Vulkan,
    pub rcx: Option<RenderContext>,
}

impl App {
    pub fn init(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let vulkan = Vulkan::init(event_loop)?;

        let state = State::default();

        let input_state = InputState::default();

        let objects = load_game_objects(&vulkan)?;

        let mut camera = Camera::new();
        camera.set_view_target(Vec3::new(1.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0));

        let mut viewer_object = GameObject::new();
        viewer_object.transform.translation.z = -2.5;

        let camera_controller = KeyboardMovementController {
            move_speed: 3.0,
            look_speed: 1.5
        };

        Ok(Self {
            state,
            input_state,
            objects,
            camera,
            viewer_object,
            camera_controller,
            vulkan,
            rcx: None,
        })
    }
}

fn load_game_objects(vulkan: &Vulkan) -> Result<Vec<GameObject>, Box<dyn Error>> {
    let model = load_model!("assets/miku.obj", vulkan.memory_allocator);
    let mut miku = GameObject::new();
    miku.model = Some(model.clone());
    miku.transform.translation = Vec3::new(-0.5, 0.5, 0.0);
    miku.transform.scale = Vec3::splat(0.1);
    miku.color = Vec4::new(0.0, 0.0, 1.0, 1.0);

    let model = load_model!("assets/link.obj", vulkan.memory_allocator);
    let mut link = GameObject::new();
    link.model = Some(model.clone());
    link.transform.translation = Vec3::new(0.5, 0.5, 0.0);
    link.transform.scale = Vec3::splat(0.06);
    link.color = Vec4::new(1.0, 0.0, 0.0, 1.0);

    let model = load_model!("assets/quad.obj", vulkan.memory_allocator);
    let mut floor = GameObject::new();
    floor.model = Some(model.clone());
    floor.transform.translation = Vec3::new(0.0, 0.5, 0.0);
    floor.transform.scale = Vec3::new(3.0, 1.0, 3.0);

    let objects = vec![floor, miku, link];

    Ok(objects)
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rcx = Some(RenderContext::init(event_loop, &self.vulkan).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // TODO: main loop
        // TODO: render

        // Event Handling
        match event {
            WindowEvent::RedrawRequested => {
                self.camera_controller.move_in_plane_xz(&self.input_state, self.rcx.as_ref().unwrap().time_info.dt, &mut self.viewer_object);
                self.camera.set_view_xyz(self.viewer_object.transform.translation, self.viewer_object.transform.rotation);
                self.draw_frame();
                self.update_time();
            }
            WindowEvent::KeyboardInput { event, .. } => self.input_state.update_keyboard_input(event),
            WindowEvent::Resized(_) => self.rcx.as_mut().unwrap().recreate_swapchain = true,
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.rcx.as_ref().unwrap().window.request_redraw();
    }
}
