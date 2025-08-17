use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct LightInfo {
    /// ambient light, first three values are rbg (between 0 and 1), last value is intensity (any)
    pub ambient_light_color: Vec4,
    pub direction_to_light: Vec3,
    pub directional_light_color: Vec4,
}
