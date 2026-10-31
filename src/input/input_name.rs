#[derive(Clone, Copy)]
pub enum InputName {
    Up,
    Down,
    Left,
    Right,
    Bomb,
    Back,
    Detonate,
}

impl InputName {
    pub fn iterator() -> impl Iterator<Item = &'static InputName> {
        [
            InputName::Up,
            InputName::Down,
            InputName::Left,
            InputName::Right,
            InputName::Bomb,
            InputName::Back,
            InputName::Detonate,
        ]
        .iter()
    }
}
