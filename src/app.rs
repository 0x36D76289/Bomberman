use crate::game::state::State;
use crate::graphics::{RenderContext, Vulkan};
use std::error::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::KeyCode,
    window::WindowId,
};

pub struct App {
    pub state: State,
    pub vulkan: Vulkan,
    pub rcx: Option<RenderContext>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            state: State::default(),
            vulkan: Vulkan::init(event_loop)?,
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
        self.state.tick();
        // TODO: render

        // Event Handling
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw_frame();
                let rcx = self.rcx.as_mut().unwrap();
                rcx.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.state.record_key(event.physical_key, event.state);
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
        self.state.fps.register_frame();
    }
}
