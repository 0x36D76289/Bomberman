use crate::app_state::{AppState, KeyMap};
use crate::game::game_state::GameState;
use crate::graphics::Graphics;
use crate::input::input::Input;
use crate::settings::settings::Settings;
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
    state_stack: Vec<AppState>,
    keys: KeyMap,
    inputs: Vec<Input>,
    graphics: Graphics,
    settings: Settings,
}

impl App {
    pub fn init(settings: Settings, event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let graphics = Graphics::new(event_loop)?;

        let keys = KeyMap::new();
        let inputs = vec![Input::default(); settings.binds.len()];

        let default_state = AppState::Game(GameState::default_state(&graphics)?);
        let state_stack = vec![default_state];

        Ok(Self {
            state_stack,
            keys,
            inputs,
            graphics,
            settings,
        })
    }

    fn record_key(&mut self, code: PhysicalKey, state: ElementState) {
        self.keys.insert(code, state);
    }

    fn update_inputs(&mut self) {
        for i in 0..self.settings.binds.len() {
            self.inputs[i].update_input_player(&self.keys, self.settings.binds[i]);
        }
    }

    fn update_state(&mut self) {
        self.update_inputs();

        let app_state = self.state_stack.last_mut().unwrap();
        let renderer = &self.graphics.renderer;

        let res = app_state.tick(
            renderer.get_delta_time(),
            &self.inputs,
            &self.keys,
            renderer.rcx().swapchain.image_extent().into(),
        );
        for _ in 0..res.1 {
            self.state_stack.pop();
        }
        if res.0.is_some() {
            self.state_stack.push(res.0.unwrap());
        }
    }

    fn render(&mut self) {
        self.graphics
            .renderer
            .render(&self.graphics.vulkan, &self.state_stack);
        self.graphics.renderer.update_time();
        self.graphics.renderer.update_title(&format!(
            "Bomberman!! fps: {:.0}",
            self.graphics.renderer.rcx().time_info.avg_fps
        ));
    }
}

impl ApplicationHandler for App {
    // This is called when the window is created
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let renderer = &mut self.graphics.renderer;
        let vulkan = &self.graphics.vulkan;

        renderer.init_render_context(event_loop, vulkan);
        renderer.create_gui(event_loop, vulkan);
        renderer.create_world_pipeline(vulkan);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.update_state();
                self.render()
            }
            WindowEvent::Resized(_) => self.graphics.renderer.recreate_swapchain(true),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => match (self.graphics.renderer.update_gui_event(&event), event) {
                (false, WindowEvent::KeyboardInput { event, .. }) => {
                    self.record_key(event.physical_key, event.state)
                }
                _ => (),
            },
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.renderer.request_redraw();
    }
}
