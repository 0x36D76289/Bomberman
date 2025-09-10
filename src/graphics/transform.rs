use glam::{Mat4, Vec3, Vec4};

use crate::input::{input::Input, input_state::InputState};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
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

    #[allow(unused)]
    pub fn keyboard_move(&mut self, input_state: &Input, delta: f32) {
        const MOVE_SPEED: f32 = 3.0;
        const LOOK_SPEED: f32 = 1.5;

        if input_state.bomb().is_down() {
            let mut rotate = Vec3::ZERO;

            if input_state.up() != InputState::Released {
                rotate.x += 1.0;
            }
            if input_state.down() != InputState::Released {
                rotate.x -= 1.0;
            }
            if input_state.right() != InputState::Released {
                rotate.y += 1.0;
            }
            if input_state.left() != InputState::Released {
                rotate.y -= 1.0;
            }

            if rotate.dot(rotate) > f32::EPSILON {
                self.rotation += LOOK_SPEED * delta * rotate.normalize()
            }

            self.rotation.x = self.rotation.x.clamp(-1.5, 1.5);
            self.rotation.y %= 2.0 * std::f32::consts::PI;
        } else {
            // if bomb is not pressed, move the camera position
            let yaw = self.rotation.y;
            let up_dir = Vec3::new(0.0, -1.0, 0.0);
            let forward_dir = Vec3::new(yaw.sin(), 0.0, yaw.cos());
            let right_dir = Vec3::new(forward_dir.z, 0.0, -forward_dir.x);

            let mut move_dir = Vec3::ZERO;

            if input_state.up() != InputState::Released {
                move_dir += forward_dir
            }
            if input_state.down() != InputState::Released {
                move_dir -= forward_dir
            }
            if input_state.right() != InputState::Released {
                move_dir += right_dir
            }
            if input_state.left() != InputState::Released {
                move_dir -= right_dir
            }
            if input_state.up() != InputState::Released
                && input_state.down() != InputState::Released
            {
                move_dir += up_dir;
            }
            if input_state.right() != InputState::Released
                && input_state.left() != InputState::Released
            {
                move_dir -= up_dir;
            }

            if move_dir.dot(move_dir) > f32::EPSILON {
                self.translation += MOVE_SPEED * delta * move_dir.normalize()
            }
        }
    }
}
