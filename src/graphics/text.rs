use glam::Vec2;

pub struct TextRenderer {
    pub characters: [Character; 128],
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Character {
    pub top_left: Vec2,
    pub top_right: Vec2,
    pub bottom_left: Vec2,
    pub bottom_right: Vec2,
}

impl TextRenderer {
    pub fn new() -> Self {
        const ATLAS_WIDTH: u32 = 126;
        const ATLAS_HEIGHT: u32 = 63;
        const CHAR_WIDTH: u32 = 7;
        const CHAR_HEIGHT: u32 = 9;
        const COL_NUMBER: u32 = ATLAS_WIDTH / CHAR_WIDTH;
        const ROW_NUBER: u32 = ATLAS_HEIGHT / CHAR_HEIGHT;

        let mut characters = [Character::default(); 128];

        for i in 0..96 {
            let row = i as u32 / COL_NUMBER;
            let col = i as u32 % COL_NUMBER;

            let character = Character {
                top_left: Vec2::new(
                    (col * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32,
                    (row * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
                ),
                top_right: Vec2::new(
                    (col * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32,
                    ((row + 1) * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
                ),
                bottom_left: Vec2::new(
                    ((col + 1) * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32,
                    (row * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
                ),
                bottom_right: Vec2::new(
                    ((col + 1) * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32,
                    ((row + 1) * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
                ),
            };

            characters[i + 32] = character;
        }

        Self { characters }
    }
}
