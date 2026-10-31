use glam::{IVec2, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Device {
    #[default]
    Keyboard,
    Mouse,
    Gamepad,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiInput {
    pub nav: IVec2,
    pub confirm: bool,
    pub back: bool,
    pub mouse: Option<Vec2>,
    pub mouse_moved: bool,
    pub mouse_pressed: bool,
    pub mouse_down: bool,
    pub mouse_released: bool,
    pub wheel: f32,
    pub device: Device,
}
