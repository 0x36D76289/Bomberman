use crate::app_state::{AppState, KeyMap};
use crate::game::game_state::GameState;
use crate::graphics::Graphics;
use std::error::Error;
use winit::event::ElementState;
use winit::keyboard::PhysicalKey;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

pub struct App {
    pub state_stack: Vec<AppState>,
    keys: KeyMap,
    pub graphics: Graphics,
}

impl App {
    pub fn init(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let graphics = Graphics::new(event_loop)?;

        let keys = KeyMap::new();

        let default_state = AppState::Game(GameState::default_state(&graphics)?);
        let state_stack = vec![default_state];

        Ok(Self {
            state_stack,
            keys,
            graphics,
        })
    }

    fn record_key(&mut self, code: PhysicalKey, state: ElementState) {
        self.keys.insert(code, state);
    }

    fn update_state(&mut self) {
        let app_state = self.state_stack.last_mut().unwrap();
        let renderer = &self.graphics.renderer;

        //TODO handle result
        app_state.tick(
            renderer.get_delta_time(),
            &self.keys,
            renderer.get_rcx().swapchain.image_extent().into(),
        );
    }

    fn draw_frame(&mut self) {
        let app_state = self.state_stack.last().unwrap();

        if let Some(mut command_buffer) = self.graphics.renderer.begin_frame(&self.graphics.vulkan)
        {
            app_state.render(&self.graphics, &mut command_buffer);

            self.graphics
                .renderer
                .end_frame(&self.graphics.vulkan, command_buffer);
            self.graphics.renderer.update_time();
            self.graphics.renderer.update_title(&format!(
                "Bomberman!! fps: {:.0}",
                self.graphics.renderer.get_rcx().time_info.avg_fps
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
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.update_state();
                self.draw_frame();
            }
            WindowEvent::Resized(_) => self.graphics.renderer.recreate_swapchain(true),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                self.record_key(event.physical_key, event.state)
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.renderer.request_redraw();
    }
}
