use glam::Vec4;

// Colors
/// Color used in backgrounds of most UIs
pub const BACKGROUND_COLOR: Vec4 = Vec4::new(0.97, 0.88, 0.96, 1.0);
/// Lightness channel of the grey button outlines
pub const OUTLINE_SHADE: f32 = 0.6;
/// Color of the button outlines
pub const OUTLINE_COLOR: Vec4 = Vec4::new(OUTLINE_SHADE, OUTLINE_SHADE, OUTLINE_SHADE, 1.0);
/// Color of the inside of buttons
pub const BUTTON_COLOR: Vec4 = Vec4::ONE;
/// The color of a selected button
pub const SELECTED_BUTTON_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
/// The standard text color
pub const TEXT_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
/// The color of text in a selected button
pub const SELECTED_TEXT_COLOR: Vec4 = Vec4::ONE;
/// The color for on screen error messages
pub const ERROR_MESSAGE_COLOR: Vec4 = Vec4::new(0.37, 0.0, 0.0, 1.0);
/// The color of text that needs to be focused on while unselected
pub const HIGHLIGHTED_TEXT_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.0, 1.0);
/// The color of the background for win screens
pub const WIN_BACKGROUND_COLOR: Vec4 = Vec4::new(0.0, 0.3, 0.0, 1.0);

// Sizes
/// The width of button outlines
pub const OUTLINE_WIDTH: f32 = 0.05;
/// The standard text width throughout the UI
pub const TEXT_SIZE: f32 = 1.0;

/// How long error messages should appear for before being entirely transparent
pub const ERROR_VISIBILITY_TIME: f32 = 4.0;
