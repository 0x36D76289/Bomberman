use crate::input::{input::Input, input_name::InputName, input_state::InputState};

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

    fn menu_up(&self) -> InputState {
        self.get_input(InputName::Up)
    }
    fn menu_down(&self) -> InputState {
        self.get_input(InputName::Down)
    }
    fn menu_left(&self) -> InputState {
        self.get_input(InputName::Left)
    }
    fn menu_right(&self) -> InputState {
        self.get_input(InputName::Right)
    }
    fn menu_confirm(&self) -> InputState {
        self.get_input(InputName::Bomb)
    }
    fn menu_back(&self) -> InputState {
        self.get_input(InputName::Back)
    }
}
