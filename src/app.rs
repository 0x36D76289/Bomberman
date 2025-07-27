use crate::game::state::State;
use crate::graphics::{Camera, GameObject, Model, RenderContext, Transform, Vulkan};
use glam::Vec3;
use std::error::Error;
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
    pub camera: Camera,
    pub objects: Vec<GameObject>,
    pub vulkan: Vulkan,
    pub rcx: Option<RenderContext>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let vulkan = Vulkan::init(event_loop)?;

        let link_model = Arc::new(Model::load(
            "src/assets/link.obj",
            vulkan.memory_allocator.clone(),
        )?);
        let miku_model = Arc::new(Model::load(
            "src/assets/miku.obj",
            vulkan.memory_allocator.clone(),
        )?);
        let link = GameObject {
            model: link_model.clone(),
            transform: Transform {
                translation: Vec3::new(-0.2, 0.0, 0.0),
                scale: Vec3::splat(0.02),
                rotation: Vec3::splat(0.0)
            },
            color: Vec3::new(1.0, 0.0, 0.0)
        };
        let miku = GameObject {
            model: miku_model.clone(),
            transform: Transform {
                translation: Vec3::new(0.2, 0.0, 0.0),
                scale: Vec3::splat(0.03),
                rotation: Vec3::splat(0.0)
            },
            color: Vec3::new(0.0, 0.0, 1.0)
        };
        let objects = vec![link, miku];

        let mut camera = Camera::new();
        camera.set_view_target(Vec3::new(0.0, 0.0, -1.0), Vec3::splat(0.0));

        Ok(Self {
            state: State::default(),
            camera,
            objects,
            vulkan,
            rcx: None,
        })
    }
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
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => self.rcx.as_mut().unwrap().recreate_swapchain = true,
            WindowEvent::RedrawRequested => {
                self.draw_frame();
                self.rcx.as_ref().unwrap().window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                println!("{event:?}");
                #[cfg(debug_assertions)]
                if event.state.is_pressed() && event.repeat == false {
                    if event.physical_key == KeyCode::Space {
                        self.state.print();
                    }
                    if event.physical_key == KeyCode::Escape {
                        event_loop.exit();
                    }
                }
            }
            _ => (),
        }
    }
}
