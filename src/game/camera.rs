use crate::{
    graphics::transform::Transform,
    input::{input::Input, input_state::InputState},
};
use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Default, Clone, Copy)]
pub struct Camera {
    pub projection_matrix: Mat4,
    pub view_matrix: Mat4,
    pub inverse_view_matrix: Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            projection_matrix: Mat4::IDENTITY,
            view_matrix: Mat4::IDENTITY,
            inverse_view_matrix: Mat4::IDENTITY,
        }
    }

    #[allow(unused)]
    pub fn set_orthographic_projection(
        &mut self,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    ) {
        self.projection_matrix = Mat4::from_cols(
            Vec4::new(2.0 / (right - left), 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 / (bottom - top), 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0 / (far - near), 0.0),
            Vec4::new(
                -(right + left) / (right - left),
                -(bottom + top) / (bottom - top),
                -near / (far - near),
                1.0,
            ),
        )
    }

    #[allow(unused)]
    pub fn set_perspective_projection(&mut self, fovy: f32, aspect: f32, near: f32, far: f32) {
        assert!((aspect - f32::EPSILON).abs() > 0.0);
        let tan_half_fovy = (fovy / 2.0).tan();
        let a = 1.0 / (aspect * tan_half_fovy);
        let b = 1.0 / tan_half_fovy;
        let c = far / (far - near);
        let d = -(far * near) / (far - near);

        self.projection_matrix = Mat4::from_cols(
            Vec4::new(a, 0.0, 0.0, 0.0),
            Vec4::new(0.0, b, 0.0, 0.0),
            Vec4::new(0.0, 0.0, c, 1.0),
            Vec4::new(0.0, 0.0, d, 0.0),
        )
    }

    #[allow(unused)]
    pub fn set_view_direction(&mut self, position: Vec3, direction: Vec3) {
        let up = Vec3::new(0.0, -1.0, 0.0);
        let w = direction.normalize();
        let u = w.cross(up).normalize();
        let v = w.cross(u);

        self.view_matrix = Mat4::from_cols(
            Vec4::new(u.x, v.x, w.x, 0.0),
            Vec4::new(u.y, v.y, w.y, 0.0),
            Vec4::new(u.z, v.z, w.z, 0.0),
            Vec4::new(-u.dot(position), -v.dot(position), -w.dot(position), 1.0),
        );

        self.inverse_view_matrix = Mat4::from_cols(
            Vec4::new(u.x, u.y, u.z, 1.0),
            Vec4::new(v.x, v.y, v.z, 1.0),
            Vec4::new(w.x, w.y, w.z, 1.0),
            Vec4::new(position.x, position.y, position.z, 1.0),
        );
    }

    #[allow(unused)]
    pub fn set_view_target(&mut self, position: Vec3, target: Vec3) {
        self.set_view_direction(position, target - position);
    }

    #[allow(unused)]
    pub fn set_view_xyz(&mut self, position: Vec3, rotation: Vec3) {
        let c3 = rotation.z.cos();
        let s3 = rotation.z.sin();
        let c2 = rotation.x.cos();
        let s2 = rotation.x.sin();
        let c1 = rotation.y.cos();
        let s1 = rotation.y.sin();

        let u = Vec3::new(c1 * c3 + s1 * s2 * s3, c2 * s3, c1 * s2 * s3 - c3 * s1);
        let v = Vec3::new(c3 * s1 * s2 - c1 * s3, c2 * c3, c1 * c3 * s2 + s1 * s3);
        let w = Vec3::new(c2 * s1, -s2, c1 * c2);

        self.view_matrix = Mat4::from_cols(
            Vec4::new(u.x, v.x, w.x, 0.0),
            Vec4::new(u.y, v.y, w.y, 0.0),
            Vec4::new(u.z, v.z, w.z, 0.0),
            Vec4::new(
                -(u.dot(position)),
                -(v.dot(position)),
                -(w.dot(position)),
                1.0,
            ),
        );

        self.inverse_view_matrix = Mat4::from_cols(
            Vec4::new(u.x, u.y, u.z, 1.0),
            Vec4::new(v.x, v.y, v.z, 1.0),
            Vec4::new(w.x, w.y, w.z, 1.0),
            Vec4::new(position.x, position.y, position.z, 1.0),
        );
    }

    #[allow(unused)]
    pub fn keyboard_move(&mut self, input_state: &Input, camera: &mut Transform, delta: f32) {
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
                camera.rotation += LOOK_SPEED * delta * rotate.normalize()
            }

            camera.rotation.x = camera.rotation.x.clamp(-1.5, 1.5);
            camera.rotation.y %= 2.0 * std::f32::consts::PI;
        } else {
            // if bomb is not pressed, move the camera position
            let yaw = camera.rotation.y;
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
                camera.translation += MOVE_SPEED * delta * move_dir.normalize()
            }
        }
    }
}
