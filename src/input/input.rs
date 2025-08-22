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

    pub fn up(&self) -> InputState {
        self.states[InputName::Up as usize]
    }
    pub fn down(&self) -> InputState {
        self.states[InputName::Down as usize]
    }
    pub fn left(&self) -> InputState {
        self.states[InputName::Left as usize]
    }
    pub fn right(&self) -> InputState {
        self.states[InputName::Right as usize]
    }
    pub fn bomb(&self) -> InputState {
        self.states[InputName::Bomb as usize]
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
pub trait GetOrDefault<T> {
    fn get_or_default(&self, i: usize) -> T;
}

impl<T: Default + Clone> GetOrDefault<T> for Vec<T> {
    fn get_or_default(&self, i: usize) -> T {
        self.get(i).cloned().unwrap_or_default()
    }
}
