use crate::input::{input::Input, input_name::InputName, input_state::InputState};

/// lets you obtain a copy of the nth element of an object or the default value
pub trait GetOrDefault<T> {
    fn get_or_default(&self, i: usize) -> T;
}

impl<T: Default + Clone> GetOrDefault<T> for Vec<T> {
    fn get_or_default(&self, i: usize) -> T {
        self.get(i).cloned().unwrap_or_default()
    }
}

pub trait MenuInput {
    fn get_input(&self, name: InputName) -> InputState;
    fn menu_up(&self) -> InputState;
    fn menu_down(&self) -> InputState;
    fn menu_left(&self) -> InputState;
    fn menu_right(&self) -> InputState;
    fn menu_confirm(&self) -> InputState;
    fn menu_back(&self) -> InputState;
}

impl MenuInput for Vec<Input> {
    /// Iterates over the vector to find the most important match of the input
    /// Prioritises [Pressed](InputState::Pressed) > [Held](InputState::Held) > [Released](InputState::Released)
    fn get_input(&self, name: InputName) -> InputState {
        let mut ret = InputState::Released;

        for input in self {
            match input.get_state(name) {
                InputState::Held => ret = InputState::Held,
                InputState::Pressed => return InputState::Pressed,
                InputState::Released => (),
            }
        }
        ret
    }

    /// Checks if any player is pressing [Up](InputName::Up)
    fn menu_up(&self) -> InputState {
        self.get_input(InputName::Up)
    }
    /// Checks if any player is pressing [Down](InputName::Down)
    fn menu_down(&self) -> InputState {
        self.get_input(InputName::Down)
    }
    /// Checks if any player is pressing [Left](InputName::Left)
    fn menu_left(&self) -> InputState {
        self.get_input(InputName::Left)
    }
    /// Checks if any player is pressing [Right](InputName::Right)
    fn menu_right(&self) -> InputState {
        self.get_input(InputName::Right)
    }
    /// Checks if any player is pressing confirm/[Bomb](InputName::Bomb)
    fn menu_confirm(&self) -> InputState {
        self.get_input(InputName::Bomb)
    }
    /// Checks if any player is pressing [Back](InputName::Back)
    fn menu_back(&self) -> InputState {
        self.get_input(InputName::Back)
    }
}
