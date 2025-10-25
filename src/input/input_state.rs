/// The various state an input can be at during any tick
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputState {
    Released,
    Pressed,
    Held,
}

impl InputState {
    /// an input is considered "down" if it is [Pressed](InputState::Pressed) or [Held](InputState::Held)
    pub fn is_down(&self) -> bool {
        !matches!(self, InputState::Released)
    }
}
