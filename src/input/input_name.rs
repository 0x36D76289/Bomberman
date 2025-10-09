use crate::{game::direction::Direction, input::input::BIND_LEN};

#[derive(Clone, Copy)]
pub enum InputName {
    Up,
    Down,
    Left,
    Right,
    Bomb,
    Back,
}

impl InputName {
    pub fn iterator() -> impl Iterator<Item = &'static InputName> {
        static DIRECTIONS: [InputName; BIND_LEN] = [
            InputName::Up,
            InputName::Down,
            InputName::Left,
            InputName::Right,
            InputName::Bomb,
            InputName::Back,
        ];
        DIRECTIONS.iter()
    }

    pub fn direction_to_input(direction: Direction) -> InputName {
        match direction {
            Direction::Down => InputName::Down,
            Direction::Up => InputName::Up,
            Direction::Right => InputName::Right,
            Direction::Left => InputName::Left,
        }
    }
}
