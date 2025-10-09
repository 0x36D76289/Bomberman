use crate::app_state::{AppState, KeyMap};
use crate::audio::{AudioManager, BackgroundMusic};
use crate::game::resources::Resources;
use crate::graphics::Graphics;
use crate::input::event::InputEvent;
use crate::input::input::Input;
use crate::settings::settings::Settings;
use crate::ui::UiState;
use crate::ui::utils::GetRatio;
use glam::Vec2;
use std::error::Error;
use winit::dpi::PhysicalPosition;
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
    mouse_pos: PhysicalPosition<f64>,
    events: Vec<InputEvent>,
    inputs: Vec<Input>,
    resources: Resources,
    graphics: Graphics,
    settings: Settings,
    audio_manager: AudioManager,
}

impl App {
    pub fn init(settings: Settings, event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let graphics = Graphics::new(event_loop)?;

        let resources = Resources::load_resources(&graphics.vulkan);

        let keys = KeyMap::new();
        let mouse_pos = Default::default();
        let events = Vec::new();
        let inputs = vec![Input::default(); settings.binds.len()];

        let gui_state1 = AppState::Ui(UiState::main_menu());

        let state_stack = vec![gui_state1];

        let mut audio_manager = AudioManager::new(&settings)?;

        audio_manager.play_background_music(BackgroundMusic::Menu);

        Ok(Self {
            state_stack,
            keys,
            inputs,
            mouse_pos,
            events,
            resources,
            graphics,
            settings,
            audio_manager,
        })
    }

    fn record_key(&mut self, code: PhysicalKey, state: ElementState) {
        self.keys.insert(code, state);
    }

    fn update_inputs(&mut self) {
        if self.inputs.len() != self.settings.binds.len() {
            self.inputs = vec![Input::held_new(); self.settings.binds.len()]
        }
        //TODO: use events
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
            &self.events,
            &self.keys,
            &self.resources,
            &mut self.audio_manager,
            &mut self.settings,
            renderer.window_size().get_ratio(),
        );
        // println!("{:#?}", self.events);
        self.events.clear();
        for _ in 0..res.1 {
            self.state_stack.pop();
            //TODO: if last state popped then handle application stop
        }
        if res.0.is_some() {
            self.state_stack.push(res.0.unwrap());
        }
    }

    fn render(&mut self) {
        self.graphics
            .renderer
            .render(&self.graphics.vulkan, &self.state_stack, &self.resources);
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

        if !renderer.is_initialized() {
            renderer.init_render_context(event_loop, vulkan);
            renderer.create_gui_pipeline(vulkan);
            renderer.create_game_pipeline(vulkan);
            renderer.create_postprocess_pipeline(vulkan);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.update_state();
                self.render()
            }
            WindowEvent::Resized(_) => self.graphics.renderer.recreate_swapchain(true),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                self.record_key(event.physical_key, event.state);
                self.events.push(InputEvent::Keyboard {
                    key: event.physical_key,
                    down: event.state.is_pressed(),
                });
            }
            WindowEvent::MouseInput {
                device_id: _,
                state: _,
                button,
            } => {
                self.events.push(InputEvent::Click {
                    location: Vec2 {
                        x: self.mouse_pos.x as f32,
                        y: self.mouse_pos.y as f32,
                    },
                    button,
                });
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.mouse_pos = position;
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.renderer.request_redraw();
    }
}
