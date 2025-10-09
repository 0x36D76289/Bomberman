use glam::Vec2;
use winit::{event::MouseButton, keyboard::PhysicalKey};

#[derive(Debug)]
pub enum InputEvent {
    Keyboard {
        key: PhysicalKey,
        down: bool,
    },
    ControllerButton {
        controller: usize,
        button: usize,
        down: bool,
    },
    Click {
        location: Vec2,
        button: MouseButton,
    },
}
