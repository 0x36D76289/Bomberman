use crate::input::{
    controller::input_events_compare, event::InputEvent, input_name::InputName,
    input_state::InputState,
};
use glam::Vec2;

pub const BIND_LEN: usize = 7;
pub type Binds = [InputEvent; BIND_LEN];

pub fn default_binds() -> Binds {
    [InputEvent::unbound(); BIND_LEN]
}

#[derive(Debug, Clone, Copy)]
pub struct Input {
    states: [InputState; BIND_LEN],
}

impl Default for Input {
    fn default() -> Self {
        Self {
            states: [InputState::Released; BIND_LEN],
        }
    }
}

impl Input {
    fn axis_to_float(negative: InputState, positive: InputState) -> f32 {
        -(negative.is_down() as u8 as f32) + positive.is_down() as u8 as f32
    }

    pub fn as_vec2(&self) -> Vec2 {
        Vec2 {
            x: Self::axis_to_float(self.left(), self.right()),
            y: Self::axis_to_float(self.down(), self.up()),
        }
    }

    pub fn get_state(&self, name: InputName) -> InputState {
        self.states[name as usize]
    }

    pub fn up(&self) -> InputState {
        self.get_state(InputName::Up)
    }
    pub fn down(&self) -> InputState {
        self.get_state(InputName::Down)
    }
    pub fn left(&self) -> InputState {
        self.get_state(InputName::Left)
    }
    pub fn right(&self) -> InputState {
        self.get_state(InputName::Right)
    }
    pub fn bomb(&self) -> InputState {
        self.get_state(InputName::Bomb)
    }
    pub fn detonate(&self) -> InputState {
        self.get_state(InputName::Detonate)
    }
    pub fn reversed(&self) -> Self {
        let mut input = *self;
        for (a, b) in [
            (InputName::Up, InputName::Down),
            (InputName::Left, InputName::Right),
        ] {
            input.states.swap(a as usize, b as usize);
        }
        input
    }
    pub fn update_input_component(&mut self, key_down: bool, input: InputName) {
        if !key_down {
            self.states[input as usize] = InputState::Released;
            return;
        }
        if self.states[input as usize].is_down() {
            self.states[input as usize] = InputState::Held;
        } else {
            self.states[input as usize] = InputState::Pressed;
        }
    }

    pub fn from_motion(motion: Vec2, bomb: bool, previous: &Input) -> Self {
        let mut input = *previous;
        input.update_input_component(motion.x < 0.0, InputName::Left);
        input.update_input_component(motion.x > 0.0, InputName::Right);
        input.update_input_component(motion.y < 0.0, InputName::Down);
        input.update_input_component(motion.y > 0.0, InputName::Up);
        input.update_input_component(bomb, InputName::Bomb);
        input.update_input_component(false, InputName::Back);
        input
    }

    pub fn update_input_player(&mut self, events: &[InputEvent], codes: &Binds) {
        for input in InputName::iterator() {
            let mut down = self.get_state(*input).is_down();
            for event in events {
                if let Some(value) = input_events_compare(event, &codes[*input as usize]) {
                    down = value;
                }
            }
            self.update_input_component(down, *input);
        }
    }
}
