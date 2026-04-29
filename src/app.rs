use crate::app_state::AppState;
use crate::audio::{AudioManager, BackgroundMusic};
use crate::game::resources::Resources;
use crate::graphics::Graphics;
use crate::input::event::InputEvent;
use crate::input::input::Input;
use crate::settings::settings::Settings;
use crate::ui::utils::GetRatio;
use gilrs::Gilrs;
use glam::Vec2;
use std::error::Error;
use winit::dpi::PhysicalPosition;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

/// The main structure containing the entire event loop
pub struct App {
    /// The list of App States, it is modified as a stack only pushing at the top or popping from
    /// the top. The state at the top of the stack is ticked every frame
    state_stack: Vec<AppState>,
    /// The last observed position of the mouse, used to save click positions
    mouse_pos: PhysicalPosition<f64>,
    /// The list of events since the last tick
    events: Vec<InputEvent>,
    /// The list of player inputs
    inputs: Vec<Input>,
    /// The app's assets
    resources: Resources,
    /// The internal graphics data
    graphics: Graphics,
    /// The application's settings, loaded at app start
    settings: Settings,
    /// The audio manager plays, interrupts, and modifies music and sound effects
    audio_manager: AudioManager,
    /// The gilrs structure polls controller events
    gilrs: Gilrs,
}

impl App {
    /// The main App constructor
    pub fn init(settings: Settings, event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let graphics = Graphics::new(event_loop)?;
        let gilrs = Gilrs::new()?;

        let resources = Resources::load_resources(&graphics.vulkan);

        let mouse_pos = Default::default();
        let events = Vec::new();
        let inputs = vec![Input::default(); settings.binds.len()];

        let gui_state1 = AppState::main_menu();

        let state_stack = vec![gui_state1];

        let mut audio_manager = AudioManager::new(&settings)?;

        audio_manager.play_background_music(BackgroundMusic::Menu);

        Ok(Self {
            state_stack,
            inputs,
            mouse_pos,
            events,
            resources,
            graphics,
            settings,
            audio_manager,
            gilrs,
        })
    }

    /// getter for the app's Resources
    pub fn get_resources(&self) -> &Resources {
        &self.resources
    }

    /// updates each of the player's inputs based on the current state of the binds
    fn update_inputs(&mut self) {
        if self.inputs.len() != self.settings.binds.len() {
            self.inputs = vec![Input::held_new(); self.settings.binds.len()]
        }
        for i in 0..self.settings.binds.len() {
            self.inputs[i].update_input_player(&self.events, self.settings.binds[i]);
        }
    }

    /// Ticks the stack's topmost element and pops or pushes according to result
    fn update_state(&mut self, event_loop: &ActiveEventLoop) {
        self.update_inputs();

        self.audio_manager.set_volume(&self.settings);

        let app_state = self.state_stack.last_mut().unwrap();
        let renderer = &self.graphics.renderer;

        let res = app_state.tick(
            renderer.get_delta_time(),
            &self.inputs,
            &self.events,
            &self.resources,
            &mut self.audio_manager,
            &mut self.settings,
            renderer.window_size().get_ratio(),
        );
        // println!("{:#?}", self.events);
        self.events.clear();
        for _ in 0..res.1 {
            if self.state_stack.pop().is_none()/* || self.state_stack.is_empty()*/ {
                event_loop.exit();
                return;
            }
        }
        if let Some(new_state) = res.0 {
            self.state_stack.push(new_state);
        }
        if self.state_stack.is_empty() {
            println!("No state in stack, exiting");
            event_loop.exit();
        }
    }

    /// Renders a single frame onto the window
    fn render(&mut self) {
        let renderer = &mut self.graphics.renderer;

        renderer.update_settings(&self.graphics.vulkan, &self.settings);
        renderer.render_states(&self.graphics.vulkan, &self.state_stack, &self.resources);
        renderer.update_time();
        renderer.update_title(&format!(
            "Bomberman!! fps: {:.0}",
            renderer.rcx().time_info.avg_fps
        ));
    }

    /// Adds every controller input to the event vector since the last tick
    fn register_controller_inputs(&mut self) {
        while let Some(event) = self.gilrs.next_event() {
            self.events.push(InputEvent::ControllerInput {
                controller: event.id,
                event: event.event,
            });
        }
    }
}

impl ApplicationHandler for App {
    // This is called when the window is created
    //TODO: doc
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let renderer = &mut self.graphics.renderer;
        let vulkan = &self.graphics.vulkan;

        if !renderer.is_initialized() {
            renderer.init_render_context(event_loop, vulkan, &self.settings);
            renderer.create_pipelines(vulkan);
        }
    }

    /// This function is called by winit on every event the window receives
    /// eg. resize, keyboard inputs, clicks, moving, etc.
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.register_controller_inputs();
                self.update_state(event_loop);
                self.render();
            }
            WindowEvent::Resized(_) => self.graphics.renderer.recreate_swapchain(true),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat {
                    return;
                }
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

    //TODO: doc
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.renderer.request_redraw();
    }
}
