use crate::game::collision::Collision;
use crate::game::map::map::Map;
use crate::game::resources::{ResourceName, Resources};
use crate::graphics::{object::Object, transform::Transform};
use glam::{Vec2, Vec3};

const ENEMY_RADIUS: f32 = 0.4;
const ENEMY_SPEED: f32 = 2.0;

#[derive(Debug, Clone)]
pub struct Enemy {
    pub position: Vec2,
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    target_pos: Vec2,
    pub alive: bool,
    pub object: Option<Object>,
}

impl Enemy {
    pub fn new(start: Vec2, end: Vec2, resources: &Resources) -> Self {
        Self {
            position: start,
            start_pos: start,
            end_pos: end,
            target_pos: end,
            alive: true,
            object: Some(Object {
                model: resources.models[&ResourceName::Bomb].clone(),
                texture: Some(resources.textures_index[&ResourceName::PowerSlide]),
                color: Vec3::new(1.0, 0.5, 0.5),
                transform: Transform {
                    translation: Vec3::new(start.x, 0.0, start.y),
                    scale: Vec3::splat(0.4),
                    rotation: Vec3::ZERO,
                },
            }),
        }
    }

    pub fn update(&mut self, delta: f32, _map: &Map) {
        if !self.alive {
            return;
        }

        if self.position.distance(self.target_pos) < 0.1 {
            self.target_pos = if self.target_pos == self.end_pos {
                self.start_pos
            } else {
                self.end_pos
            };
        }

        let direction = (self.target_pos - self.position).normalize_or_zero();
        let motion = direction * delta * ENEMY_SPEED;

        self.position += motion;

        if let Some(obj) = &mut self.object {
            obj.transform.translation.x = self.position.x;
            obj.transform.translation.z = self.position.y;
        }
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.object = None;
    }
}

impl Collision for Enemy {
    fn get_pos(&self) -> Vec2 {
        self.position
    }
    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos;
    }
    fn get_size(&self) -> f32 {
        ENEMY_RADIUS
    }
}
