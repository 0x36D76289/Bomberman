use crate::game::direction::Direction;

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
    
    pub fn direction_to_input(direction: Direction) -> InputName {
        match direction {
            Direction::Down => InputName::Down,
            Direction::Up => InputName::Up,
            Direction::Right => InputName::Right,
            Direction::Left => InputName::Left,    
        }
    }
}
