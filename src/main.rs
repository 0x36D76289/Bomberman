mod game;

use game::state::State;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    rcx: Option<RenderContext>,
    state: State,
}

pub struct RenderContext {
    window: Arc<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Bomberman!"))
                .unwrap(),
        );
        self.rcx = Some(RenderContext { window });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // TODO: main loop
        // TODO: render

        // Event Handling
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let rcx = self.rcx.as_mut().unwrap();
                rcx.window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } =>
            {
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

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
