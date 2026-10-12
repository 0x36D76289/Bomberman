use crate::app_state::{AppState, Ctx, POP_TO_MENU, Transition};
use crate::audio::{AudioManager, SoundEffect};
use crate::game::resources::Resources;
use crate::graphics::Graphics;
use crate::graphics::text::FontId;
use crate::input::controller::is_bindable_action;
use crate::input::event::InputEvent;
use crate::input::input::Input;
use crate::input::input_state::InputState;
use crate::input::input_vec::MenuInput;
use crate::settings::settings::Settings;
use crate::ui::UiState;
use crate::ui::draw::{DrawList, TextStyle};
use crate::ui::input::{Device, UiInput};
use crate::ui::layout::{Rect, screen_size};
use crate::ui::style;
use crate::ui::textures::{Icon, UiTex};
use anyhow::{Result, anyhow};
use gilrs::{EventType, Gilrs};
use glam::{IVec2, Vec2, Vec4};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

#[derive(Debug, Default)]
struct Mouse {
    position: Option<PhysicalPosition<f64>>,
    moved: bool,
    pressed: bool,
    down: bool,
    released: bool,
    wheel: f32,
}

pub struct App {
    state_stack: Vec<AppState>,
    events: Vec<InputEvent>,
    inputs: Vec<Input>,
    resources: &'static Resources,
    graphics: Graphics,
    settings: Settings,
    audio_manager: AudioManager,
    gilrs: Gilrs,
    last_frame: Instant,
    mouse: Mouse,
    device: Device,
    nav_repeat: (IVec2, f32),
    toast: Option<(String, f32)>,
    overlay: UiState,
    cursor_visible: bool,
}

impl App {
    pub fn init(
        settings: Settings,
        event_loop: &EventLoop<()>,
        resources: &'static OnceLock<Resources>,
    ) -> Result<Self> {
        let graphics = Graphics::new(event_loop).map_err(|e| anyhow!("{e}"))?;

        let loaded = Resources::load_resources(&graphics.vulkan)?;
        let resources = resources.get_or_init(|| loaded);

        let mut audio_manager = AudioManager::new(&settings)?;
        let state_stack = vec![AppState::main_menu()];
        if let Some(top) = state_stack.last() {
            top.update_music(&mut audio_manager);
        }

        Ok(Self {
            state_stack,
            inputs: Vec::new(),
            events: Vec::new(),
            resources,
            graphics,
            settings,
            audio_manager,
            gilrs: Gilrs::new().map_err(|e| anyhow!("{e}"))?,
            last_frame: Instant::now(),
            mouse: Mouse::default(),
            device: Device::Keyboard,
            nav_repeat: (IVec2::ZERO, 0.0),
            toast: None,
            overlay: UiState::default(),
            cursor_visible: true,
        })
    }

    fn update_inputs(&mut self) {
        let count = self.settings.binds.len().clamp(1, 8);
        if self.inputs.len() != count {
            self.inputs.resize(count, Input::default());
        }
        for (input, binds) in self.inputs.iter_mut().zip(&self.settings.binds) {
            input.update_input_player(&self.events, binds);
        }
    }

    fn ui_input(&mut self, delta: f32, screen: Vec2) -> UiInput {
        let inputs = &self.inputs;
        let held = |state: InputState| state.is_down();
        let dir = IVec2::new(
            held(inputs.menu_right()) as i32 - held(inputs.menu_left()) as i32,
            held(inputs.menu_down()) as i32 - held(inputs.menu_up()) as i32,
        );
        let mut nav = IVec2::ZERO;
        if dir != self.nav_repeat.0 {
            nav = dir;
            self.nav_repeat = (dir, 0.35);
        } else if dir != IVec2::ZERO {
            self.nav_repeat.1 -= delta;
            if self.nav_repeat.1 <= 0.0 {
                nav = dir;
                self.nav_repeat.1 = 0.09;
            }
        }
        if nav.x != 0 && nav.y != 0 {
            nav.x = 0;
        }

        let window = self.graphics.renderer.window_size();
        UiInput {
            nav,
            confirm: inputs.menu_confirm() == InputState::Pressed
                || self.events.iter().any(|e| {
                    matches!(
                        e,
                        InputEvent::Keyboard {
                            key: PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter),
                            down: true
                        }
                    )
                }),
            back: inputs.menu_back() == InputState::Pressed,
            mouse: self
                .mouse
                .position
                .map(|p| {
                    Vec2::new(
                        p.x as f32 / window[0].max(1) as f32 * screen.x,
                        p.y as f32 / window[1].max(1) as f32 * screen.y,
                    )
                })
                .filter(|_| self.cursor_visible),
            mouse_moved: self.mouse.moved,
            mouse_pressed: self.mouse.pressed,
            mouse_down: self.mouse.down,
            mouse_released: self.mouse.released,
            wheel: self.mouse.wheel,
            device: self.device,
        }
    }

    fn apply(&mut self, (new_state, pop): Transition) -> bool {
        if pop == POP_TO_MENU {
            self.state_stack.truncate(1);
            self.state_stack[0] = AppState::main_menu();
        } else {
            let pop = pop as usize;

            if pop >= self.state_stack.len() && new_state.is_none() {
                return false;
            }
            self.state_stack
                .truncate(self.state_stack.len().saturating_sub(pop));
            if self.state_stack.is_empty() {
                self.state_stack.push(AppState::main_menu());
            }
            if pop > 0
                && let Some(top) = self.state_stack.last_mut()
                && !top.is_gameplay()
                && let Some(ui) = top.ui.as_mut()
            {
                ui.restart();
            }
        }
        if let Some(new_state) = new_state {
            self.state_stack.push(new_state);
        }
        if let Some(top) = self.state_stack.last() {
            top.update_music(&mut self.audio_manager);
        }
        true
    }

    fn update_state(&mut self, event_loop: &ActiveEventLoop) {
        self.update_inputs();
        self.audio_manager.set_volume(&self.settings);

        let renderer = &self.graphics.renderer;
        let delta = renderer.get_delta_time().min(0.1);
        let window = renderer.window_size();
        let screen = screen_size(window[0] as f32 / window[1].max(1) as f32);
        let ui_input = self.ui_input(delta, screen);

        let Some(app_state) = self.state_stack.last_mut() else {
            event_loop.exit();
            return;
        };
        let result = app_state.tick(&mut Ctx {
            delta,
            inputs: &self.inputs,
            events: &self.events,
            ui: &ui_input,
            resources: self.resources,
            audio: &mut self.audio_manager,
            settings: &mut self.settings,
            screen,
        });

        self.events.clear();
        self.mouse.moved = false;
        self.mouse.pressed = false;
        self.mouse.released = false;
        self.mouse.wheel = 0.0;

        if (result.0.is_some() || result.1 != 0) && !self.apply(result) {
            event_loop.exit();
            return;
        }

        let show_cursor = !self.state_stack.last().is_some_and(|s| s.is_gameplay());
        if show_cursor != self.cursor_visible {
            self.cursor_visible = show_cursor;
            self.graphics
                .renderer
                .rcx()
                .window
                .set_cursor_visible(show_cursor);
        }
        self.draw_overlay(delta, screen);
    }

    fn draw_overlay(&mut self, delta: f32, screen: Vec2) {
        let mut dl = DrawList::new(screen, self.resources);
        if let Some((text, time)) = &mut self.toast {
            *time += delta;
            let rect = Rect::new(screen.x - 560.0, 30.0, 520.0, 90.0);
            dl.frame(UiTex::Panel, rect, Vec4::ONE, 0.45);
            dl.image_fit(
                UiTex::Icon(Icon::Gamepad),
                Rect::new(rect.x + 24.0, rect.y + 18.0, 54.0, 54.0),
                style::rgb(0x3c, 0xbc, 0xfc),
            );
            dl.text(
                text,
                Vec2::new(rect.x + 100.0, rect.center().y),
                TextStyle::new(FontId::Title, 30.0, style::TEXT_DARK)
                    .align(crate::ui::draw::Align::Left),
            );
            if *time > 2.5 {
                self.toast = None;
            }
        }
        self.overlay.verts = dl.verts;
    }

    fn render(&mut self) {
        let renderer = &mut self.graphics.renderer;

        renderer.update_settings(&self.graphics.vulkan, &self.settings);
        renderer.render_states(
            &self.graphics.vulkan,
            &self.state_stack,
            self.resources,
            (!self.overlay.verts.is_empty()).then_some(&self.overlay),
        );
        renderer.update_time();

        if self.settings.framerate_limit > 0 {
            let frame = Duration::from_secs_f64(1.0 / self.settings.framerate_limit as f64);
            let elapsed = self.last_frame.elapsed();
            if elapsed < frame {
                std::thread::sleep(frame - elapsed);
            }
        }
        self.last_frame = Instant::now();
    }

    fn show_toast(&mut self, text: &str) {
        if !self.toast.as_ref().is_some_and(|(shown, _)| shown == text) {
            self.toast = Some((text.to_string(), 0.0));
        }
    }

    fn register_controller_inputs(&mut self) {
        while let Some(event) = self.gilrs.next_event() {
            match event.event {
                EventType::Connected => {
                    self.show_toast("Manette connectée");
                    self.audio_manager
                        .play_sound_effect(SoundEffect::MenuConfirm);
                }
                EventType::Disconnected => {
                    self.show_toast("Manette déconnectée");
                    self.audio_manager.play_sound_effect(SoundEffect::MenuBack);
                }
                _ => (),
            }
            let event = InputEvent::ControllerInput {
                controller: event.id,
                event: event.event,
            };
            if is_bindable_action(&event) {
                self.device = Device::Gamepad;
            }
            self.events.push(event);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let renderer = &mut self.graphics.renderer;
        let vulkan = &self.graphics.vulkan;

        if !renderer.is_initialized() {
            renderer.init_render_context(event_loop, vulkan, &self.settings);
            renderer.create_pipelines(vulkan);
        }
    }

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
                self.device = Device::Keyboard;
                self.events.push(InputEvent::Keyboard {
                    key: event.physical_key,
                    down: event.state.is_pressed(),
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.device = Device::Mouse;
                if button == MouseButton::Left {
                    let pressed = state == ElementState::Pressed;
                    self.mouse.pressed |= pressed;
                    self.mouse.released |= !pressed;
                    self.mouse.down = pressed;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.device = Device::Mouse;
                self.mouse.wheel += match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => (p.y / 40.0) as f32,
                };
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self
                    .mouse
                    .position
                    .is_none_or(|old| (old.x - position.x).abs() + (old.y - position.y).abs() > 2.0)
                {
                    self.device = Device::Mouse;
                    self.mouse.moved = true;
                }
                self.mouse.position = Some(position);
            }
            WindowEvent::CursorLeft { .. } => self.mouse.position = None,
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.renderer.request_redraw();
    }
}
