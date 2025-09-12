use glam::Vec4;

// Colors
pub const BACKGROUND_COLOR: Vec4 = Vec4::new(0.97, 0.88, 0.96, 1.0);
pub const OUTLINE_SHADE: f32 = 0.6;
pub const OUTLINE_COLOR: Vec4 = Vec4::new(OUTLINE_SHADE, OUTLINE_SHADE, OUTLINE_SHADE, 1.0);
pub const BUTTON_COLOR: Vec4 = Vec4::ONE;
pub const SELECTED_BUTTON_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
pub const TEXT_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
pub const SELECTED_TEXT_COLOR: Vec4 = Vec4::ONE;
pub const ERROR_MESSAGE_COLOR: Vec4 = Vec4::new(0.37, 0.0, 0.0, 1.0);

// Sizes
pub const OUTLINE_WIDTH: f32 = 0.05;
pub const TEXT_SIZE: f32 = 1.0;

pub const ERROR_VISIBILITY_TIME: f32 = 4.0;
