use glam::Vec2;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputState {
    Released,
    Pressed,
    Held,
}

#[derive(Debug, Clone, Copy)]
pub struct Input {
    pub up: InputState,
    pub down: InputState,
    pub left: InputState,
    pub right: InputState,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            up: InputState::Released,
            down: InputState::Released,
            left: InputState::Released,
            right: InputState::Released,
        }
    }
}

impl Input {
    fn axis_to_float(negative: InputState, positive: InputState) -> f32 {
        if negative == InputState::Released && positive == InputState::Released {
            return 0.0;
        }
        if positive == InputState::Pressed || positive == InputState::Held {
            return 1.0;
        }
        return -1.0;
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: Self::axis_to_float(self.left, self.right),
            y: Self::axis_to_float(self.up, self.down),
        }
    }
}
