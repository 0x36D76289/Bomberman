use glam::Vec2;
use std::collections::HashMap;
use std::slice::Iter;
use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputState {
    Released,
    Pressed,
    Held,
}

impl InputState {
    fn is_down(&self) -> bool {
        return match self {
            InputState::Released => false,
            _ => true,
        };
    }
}

#[derive(Clone, Copy)]
pub enum InputName {
    Up,
    Down,
    Left,
    Right,
    Bomb,
}

impl InputName {
    pub fn iterator() -> impl Iterator<Item = &'static InputName> {
        static DIRECTIONS: [InputName; 5] = [
            InputName::Up,
            InputName::Down,
            InputName::Left,
            InputName::Right,
            InputName::Bomb,
        ];
        DIRECTIONS.iter()
    }

    pub fn value(&self) -> usize {
        match self {
            InputName::Up => 0,
            InputName::Down => 1,
            InputName::Left => 2,
            InputName::Right => 3,
            InputName::Bomb => 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Input {
    states: [InputState; 5],
}

impl Default for Input {
    fn default() -> Self {
        Self {
            states: [InputState::Released; 5],
        }
    }
}

impl Input {
    fn axis_to_float(negative: InputState, positive: InputState) -> f32 {
        negative.is_down() as u8 as f32 * -1.0 + positive.is_down() as u8 as f32 * 1.0
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: Self::axis_to_float(self.left(), self.right()),
            y: Self::axis_to_float(self.up(), self.down()),
        }
    }

    pub fn up(&self) -> InputState {
        self.states[InputName::Up.value()]
    }
    pub fn down(&self) -> InputState {
        self.states[InputName::Down.value()]
    }
    pub fn left(&self) -> InputState {
        self.states[InputName::Left.value()]
    }
    pub fn right(&self) -> InputState {
        self.states[InputName::Right.value()]
    }
    pub fn bomb(&self) -> InputState {
        self.states[InputName::Bomb.value()]
    }

    pub fn update_input(&mut self, key_down: bool, input: InputName) {
        if !key_down {
            self.states[input.value()] = InputState::Released;
            return;
        }
        if self.states[input.value()].is_down() {
            self.states[input.value()] = InputState::Held;
        } else {
            self.states[input.value()] = InputState::Pressed;
        }
    }

    fn update_input_keycode(
        &mut self,
        map: &HashMap<PhysicalKey, ElementState>,
        key: KeyCode,
        input: InputName,
    ) {
        match map.get(&PhysicalKey::Code(key)) {
            Some(state) => {
                self.update_input(state.is_pressed(), input);
            }
            None => (),
        }
    }

    pub fn update_input_player(
        &mut self,
        map: &HashMap<PhysicalKey, ElementState>,
        codes: [KeyCode; 5],
    ) {
        for input in InputName::iterator() {
            self.update_input_keycode(map, codes[input.value()], input.clone());
        }
    }
}
