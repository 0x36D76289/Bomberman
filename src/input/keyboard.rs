use crate::{game::Entity, input::InputState};
use glam::Vec3;

#[derive(Debug, Clone, Copy, Default)]
pub struct KeyboardMovementController {
    pub move_speed: f32,
    pub look_speed: f32,
}

impl KeyboardMovementController {
    pub fn move_in_plane_xz(&self, input_state: &InputState, dt: f32, entity: &mut Entity) {
        let transform = &mut entity.physics.as_mut().unwrap().transform;

        let mut rotate = Vec3::splat(0.0);

        if input_state.look_right {
            rotate.y += 1.0
        }
        if input_state.look_left {
            rotate.y -= 1.0
        }
        if input_state.look_up {
            rotate.x += 1.0
        }
        if input_state.look_down {
            rotate.x -= 1.0
        }

        if rotate.dot(rotate) > f32::EPSILON {
            transform.rotation += self.look_speed * dt * rotate.normalize()
        }

        transform.rotation.x = transform.rotation.x.clamp(-1.5, 1.5);
        transform.rotation.y = transform.rotation.y % (2.0 * std::f32::consts::PI);

        let yaw = transform.rotation.y;
        let forward_dir = Vec3::new(yaw.sin(), 0.0, yaw.cos());
        let right_dir = Vec3::new(forward_dir.z, 0.0, -forward_dir.x);
        let up_dir = Vec3::new(0.0, -1.0, 0.0);

        let mut move_dir = Vec3::splat(0.0);

        if input_state.move_forward {
            move_dir += forward_dir
        }
        if input_state.move_backward {
            move_dir -= forward_dir
        }
        if input_state.move_right {
            move_dir += right_dir
        }
        if input_state.move_left {
            move_dir -= right_dir
        }
        if input_state.move_up {
            move_dir += up_dir
        }
        if input_state.move_down {
            move_dir -= up_dir
        }

        if move_dir.dot(move_dir) > f32::EPSILON {
            transform.translation += self.move_speed * dt * move_dir.normalize()
        }
    }

    pub fn move_in_plane_xz_player(&self, input_state: &InputState, dt: f32, entity: &mut Entity) {
        let transform = &mut entity.physics.as_mut().unwrap().transform;

        // let mut rotate = Vec3::splat(0.0);

        // if input_state.look_right {
        //     rotate.y += 1.0
        // }
        // if input_state.look_left {
        //     rotate.y -= 1.0
        // }
        // if input_state.look_up {
        //     rotate.x += 1.0
        // }
        // if input_state.look_down {
        //     rotate.x -= 1.0
        // }

        // if rotate.dot(rotate) > f32::EPSILON {
        //     transform.rotation += self.look_speed * dt * rotate.normalize()
        // }

        // transform.rotation.x = transform.rotation.x.clamp(-1.5, 1.5);
        // transform.rotation.y = transform.rotation.y % (2.0 * std::f32::consts::PI);

        let forward_dir = Vec3::new(0.0, 0.0, 1.0);
        let right_dir = Vec3::new(1.0, 0.0, 0.0);
        let up_dir = Vec3::new(0.0, -1.0, 0.0);

        let mut move_dir = Vec3::splat(0.0);

        if input_state.move_forward {
            move_dir += forward_dir
        }
        if input_state.move_backward {
            move_dir -= forward_dir
        }
        if input_state.move_right {
            move_dir += right_dir
        }
        if input_state.move_left {
            move_dir -= right_dir
        }
        if input_state.move_up {
            move_dir += up_dir
        }
        if input_state.move_down {
            move_dir -= up_dir
        }

        if move_dir.dot(move_dir) > f32::EPSILON {
            move_dir = move_dir.normalize();
            transform.translation += self.move_speed * dt * move_dir;
            transform.rotation.y = move_dir.x.atan2(move_dir.z);
        }
    }
}
