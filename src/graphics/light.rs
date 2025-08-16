use glam::{Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct LightInfo {
    pub ambient_light_color: Vec4,
    pub direction_to_light: Vec3,
    pub directional_light_color: Vec4,
}
