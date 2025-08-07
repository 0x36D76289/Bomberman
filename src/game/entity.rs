use crate::graphics::Model;
use glam::{Mat4, Vec3, Vec4};
use std::sync::Arc;

type TextureIndex = i32;
type LightIntensity = f32;

#[derive(Debug, Clone, Default)]
pub struct Entity {
    pub name: Option<String>,
    pub model: Option<Arc<Model>>,
    pub texture: Option<TextureIndex>,
    pub physics: Option<Physics>,
    pub light: Option<LightIntensity>,
    pub color: Option<Vec3>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Physics {
    pub transform: Transform,
    pub velocity: Vec3,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Transform {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
}

impl Entity {
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_model(mut self, model: Arc<Model>) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_texture(mut self, texture: TextureIndex) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_physics(mut self, physics: Physics) -> Self {
        self.physics = Some(physics);
        self
    }

    pub fn with_position(self, position: Vec3) -> Self {
        let mut physics = self.physics.unwrap_or_default();
        physics.transform.translation = position;
        self.with_physics(physics)
    }

    pub fn with_rotation(self, rotation: Vec3) -> Self {
        let mut physics = self.physics.unwrap_or_default();
        physics.transform.rotation = rotation;
        self.with_physics(physics)
    }

    pub fn with_scale(self, scale: Vec3) -> Self {
        let mut physics = self.physics.unwrap_or_default();
        physics.transform.scale = scale;
        self.with_physics(physics)
    }

    pub fn with_light(mut self, light: LightIntensity) -> Self {
        self.light = Some(light);
        self
    }

    pub fn with_color(mut self, color: Vec3) -> Self {
        self.color = Some(color);
        self
    }
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
            Vec4::new(
                self.scale.x * (c1 * c3 + s1 * s2 * s3),
                self.scale.x * (c2 * s3),
                self.scale.x * (c1 * s2 * s3 - c3 * s1),
                0.0,
            ),
            Vec4::new(
                self.scale.y * (c3 * s1 * s2 - c1 * s3),
                self.scale.y * (c2 * c3),
                self.scale.y * (c1 * c3 * s2 + s1 * s3),
                0.0,
            ),
            Vec4::new(
                self.scale.z * (c2 * s1),
                self.scale.z * (-s2),
                self.scale.z * (c1 * c2),
                0.0,
            ),
            Vec4::new(
                self.translation.x,
                self.translation.y,
                self.translation.z,
                1.0,
            ),
        )
    }

    pub fn normal_matrix(&self) -> Mat4 {
        let c3 = self.rotation.z.cos();
        let s3 = self.rotation.z.sin();
        let c2 = self.rotation.x.cos();
        let s2 = self.rotation.x.sin();
        let c1 = self.rotation.y.cos();
        let s1 = self.rotation.y.sin();
        let inv_scale = 1.0 / self.scale;

        Mat4::from_cols(
            Vec4::new(
                inv_scale.x * (c1 * c3 + s1 * s2 * s3),
                inv_scale.x * (c2 * s3),
                inv_scale.x * (c1 * s2 * s3 - c3 * s1),
                0.0,
            ),
            Vec4::new(
                inv_scale.y * (c3 * s1 * s2 - c1 * s3),
                inv_scale.y * (c2 * c3),
                inv_scale.y * (c1 * c3 * s2 + s1 * s3),
                0.0,
            ),
            Vec4::new(
                inv_scale.z * (c2 * s1),
                inv_scale.z * (-s2),
                inv_scale.z * (c1 * c2),
                0.0,
            ),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }
}
