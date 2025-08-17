#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputState {
    Released,
    Pressed,
    Held,
}

impl InputState {
    pub fn is_down(&self) -> bool {
        return match self {
            InputState::Released => false,
            _ => true,
        };
    }
}
