use glam::Vec2;

use crate::input::{
    controller::input_events_compare, event::InputEvent, input_name::InputName,
    input_state::InputState,
};

/// The amount of binds for each player
pub const BIND_LEN: usize = 6;
/// The binds of each player
pub type Binds = [InputEvent; BIND_LEN];

/// Default constructor for [Binds] as it is a foreign type (can't impl)
pub fn default_binds() -> Binds {
    [InputEvent::unbound(); BIND_LEN]
}

/// The current state of a player's input
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
    /// Constructor with every key already pressed, used when changing player count
    pub fn held_new() -> Self {
        Self {
            states: [InputState::Held; BIND_LEN],
        }
    }

    /// Converts 2 directions into a single value from [-1.0] to [1.0]
    fn axis_to_float(negative: InputState, positive: InputState) -> f32 {
        -(negative.is_down() as u8 as f32) + positive.is_down() as u8 as f32
    }

    /// Creates a Vec2 representing the position of the 4 movement keys
    pub fn as_vec2(&self) -> Vec2 {
        Vec2 {
            x: Self::axis_to_float(self.left(), self.right()),
            y: Self::axis_to_float(self.down(), self.up()),
        }
    }

    /// Query a single InputState from a player
    pub fn get_state(&self, name: InputName) -> InputState {
        self.states[name as usize]
    }

    /// Gets the current value of a player's Up input
    pub fn up(&self) -> InputState {
        self.get_state(InputName::Up)
    }
    /// Gets the current value of a player's Down input
    pub fn down(&self) -> InputState {
        self.get_state(InputName::Down)
    }
    /// Gets the current value of a player's Left input
    pub fn left(&self) -> InputState {
        self.get_state(InputName::Left)
    }
    /// Gets the current value of a player's Right input
    pub fn right(&self) -> InputState {
        self.get_state(InputName::Right)
    }
    /// Gets the current value of a player's Bomb input
    pub fn bomb(&self) -> InputState {
        self.get_state(InputName::Bomb)
    }
    /// Gets the current value of a player's Back input
    pub fn back(&self) -> InputState {
        self.get_state(InputName::Back)
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

    // TODO: remove or document (unused)
    pub fn release_all(&mut self) {
        for state in self.states.iter_mut() {
            *state = InputState::Released;
        }
    }

    // TODO: remove or document (unused)
    pub fn release_all_but(&mut self, input: InputName) {
        for (i, state) in self.states.iter_mut().enumerate() {
            if i != input as usize {
                *state = InputState::Released;
            }
        }
    }

    /// Updates a single Input State from a player's binds and the tick's events
    fn update_input_component_from_events(
        &mut self,
        events: &Vec<InputEvent>,
        bind: &InputEvent,
        input: InputName,
    ) {
        let mut state = self.states[input as usize].is_down();

        for event in events {
            if let Some(value) = input_events_compare(event, bind) {
                state = value;
            }
        }
        self.update_input_component(state, input);
    }

    /// Updates all of a player's input by using their keybinds
    pub fn update_input_player(&mut self, events: &Vec<InputEvent>, codes: Binds) {
        for input in InputName::iterator() {
            self.update_input_component_from_events(events, &codes[*input as usize], *input);
        }
    }
}
