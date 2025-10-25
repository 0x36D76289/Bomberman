use crate::{game::direction::Direction, input::input::BIND_LEN};

/// The list of inputs a player can emit
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
    /// Iterates over all of the enum's values
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

    // TODO: document or remove (unused)
    pub fn direction_to_input(direction: Direction) -> InputName {
        match direction {
            Direction::Down => InputName::Down,
            Direction::Up => InputName::Up,
            Direction::Right => InputName::Right,
            Direction::Left => InputName::Left,
        }
    }
}
