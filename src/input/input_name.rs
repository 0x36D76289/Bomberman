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
}
