use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Default, Clone, Copy)]
pub struct Camera {
    pub projection_matrix: Mat4,
    pub view_matrix: Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            projection_matrix: Mat4::IDENTITY,
            view_matrix: Mat4::IDENTITY,
        }
    }

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
    }
}
