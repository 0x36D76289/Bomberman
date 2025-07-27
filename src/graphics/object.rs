use crate::graphics::Model;
use glam::{Mat4, Vec3, Vec4};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GameObject {
    pub model: Arc<Model>,
    pub transform: Transform,
    pub color: Vec3
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3
}

impl Transform {
    // Matrix corrsponds to Translate * Ry * Rx * Rz * Scale
    // Rotations correspond to Tait-bryan angles of Y(1), X(2), Z(3)
    // https://en.wikipedia.org/wiki/Euler_angles#Rotation_matrix
    pub fn mat4(&self) -> Mat4 {
        let c3 = self.rotation.z.cos();
        let s3 = self.rotation.z.sin();
        let c2 = self.rotation.x.cos();
        let s2 = self.rotation.x.sin();
        let c1 = self.rotation.y.cos();
        let s1 = self.rotation.y.sin();

        Mat4::from_cols(
            Vec4::new(self.scale.x * (c1 * c3 + s1 * s2 * s3), self.scale.x * (c2 * s3), self.scale.x * (c1 * s2 * s3 - c3 * s1), 0.0),
            Vec4::new(self.scale.y * (c3 * s1 * s2 - c1 * s3), self.scale.y * (c2 * c3), self.scale.y * (c1 * c3 * s2 + s1 * s3), 0.0),
            Vec4::new(self.scale.z * (c2 * s1), self.scale.z * (-s2), self.scale.z * (c1 * c2), 0.0),
            Vec4::new(self.translation.x, self.translation.y, self.translation.z, 1.0),
        )
    }
}
