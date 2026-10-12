use glam::Vec4;

pub const fn rgb(r: u8, g: u8, b: u8) -> Vec4 {
    Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

pub const INK: Vec4 = rgb(0x00, 0x00, 0x00);
pub const TEXT_DARK: Vec4 = rgb(0xfc, 0xfc, 0xfc);
pub const TEXT_MUTED: Vec4 = rgb(0xa4, 0xa4, 0xa4);
pub const WHITE: Vec4 = rgb(0xfc, 0xfc, 0xfc);
pub const YELLOW: Vec4 = rgb(0xf8, 0xb8, 0x00);
pub const RED: Vec4 = rgb(0xd8, 0x28, 0x00);
pub const GREEN: Vec4 = rgb(0x00, 0xa8, 0x00);
pub const GREY: Vec4 = rgb(0x5c, 0x5c, 0x5c);
pub const PANEL_GREY: Vec4 = rgb(0x24, 0x24, 0x30);
pub const OVERLAY: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.7);
pub const BACKDROP_TOP: Vec4 = rgb(0x0f, 0x0f, 0x23);
pub const BACKDROP_BOTTOM: Vec4 = rgb(0x00, 0x00, 0x00);
pub const PIXEL: f32 = 4.0;
pub const HEADER: f32 = 64.0;
pub const TEXT: f32 = 30.0;
pub const MARGIN: f32 = 48.0;

pub fn fade(color: Vec4, alpha: f32) -> Vec4 {
    color.with_w(color.w * alpha)
}
