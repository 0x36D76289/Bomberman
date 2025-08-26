use glam::Vec2;
use std::collections::HashMap;
use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::input::{input_name::InputName, input_state::InputState};

pub type Binds = [KeyCode; 5];

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

    /// Updates an individual input component
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

    pub fn release_all(&mut self) {
        for state in self.states.iter_mut() {
            *state = InputState::Released;
        }
    }

    pub fn release_all_but(&mut self, input: InputName) {
        for (i, state) in self.states.iter_mut().enumerate() {
            if i != input as usize {
                *state = InputState::Released;
            }
        }
    }

    fn update_input_keycode(
        &mut self,
        map: &HashMap<PhysicalKey, ElementState>,
        key: KeyCode,
        input: InputName,
    ) {
        if let Some(state) = map.get(&PhysicalKey::Code(key)) {
            self.update_input_component(state.is_pressed(), input);
        }
    }

    /// Updates all of a player's input by using their keybinds
    pub fn update_input_player(&mut self, map: &HashMap<PhysicalKey, ElementState>, codes: Binds) {
        for input in InputName::iterator() {
            self.update_input_keycode(map, codes[*input as usize], *input);
        }
    }
}
