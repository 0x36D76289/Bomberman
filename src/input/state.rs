use winit::{
    event::KeyEvent,
    keyboard::{Key, NamedKey},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct InputState {
    pub move_left: bool,
    pub move_right: bool,
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub look_right: bool,
    pub look_left: bool,
    pub look_up: bool,
    pub look_down: bool,
}

impl InputState {
    pub fn update_keyboard_input(&mut self, event: KeyEvent) {
        let state = event.state.is_pressed();

        match event.logical_key.as_ref() {
            Key::Character("w") => self.move_forward = state,
            Key::Character("s") => self.move_backward = state,
            Key::Character("a") => self.move_left = state,
            Key::Character("d") => self.move_right = state,
            Key::Named(NamedKey::Space) => self.move_up = state,
            Key::Named(NamedKey::Control) => self.move_down = state,
            Key::Named(NamedKey::ArrowLeft) => self.look_left = state,
            Key::Named(NamedKey::ArrowRight) => self.look_right = state,
            Key::Named(NamedKey::ArrowUp) => self.look_up = state,
            Key::Named(NamedKey::ArrowDown) => self.look_down = state,
            _ => (),
        }
    }
}
