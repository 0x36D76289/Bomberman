use crate::app_state::{AppState, CommandBuffer, KeyMap};
use crate::game::game_state::GameState;
use crate::graphics::Graphics;
use crate::input::input::Input;
use crate::settings::settings::Settings;
use glam::usize;
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
            renderer.get_rcx().swapchain.image_extent().into(),
        );
        for _ in 0..res.1 {
            self.state_stack.pop();
        }
        if res.0.is_some() {
            self.state_stack.push(res.0.unwrap());
        }
    }

    fn render_state(&self, command_buffer: &mut CommandBuffer, pos: usize) {
        if self.state_stack[pos].is_transparent() && pos != 0 {
            self.render_state(command_buffer, pos - 1);
        }
        self.state_stack[pos].render(&self.graphics, command_buffer);
    }

    fn draw_frame(&mut self) {
        if let Some(mut command_buffer) = self.graphics.renderer.begin_frame(&self.graphics.vulkan)
        {
            self.render_state(&mut command_buffer, self.state_stack.len() - 1);
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
