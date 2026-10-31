use crate::game::map::map::Map;
use glam::Vec2;

pub trait Collision {
    fn get_pos(&self) -> Vec2;
    fn set_pos(&mut self, pos: Vec2);
    fn get_size(&self) -> f32;

    fn bound(&mut self, map: &Map) {
        let mut pos = self.get_pos();
        pos.x = pos
            .x
            .clamp(self.get_size(), map.width as f32 - self.get_size());
        pos.y = pos
            .y
            .clamp(self.get_size(), map.height as f32 - self.get_size());
        self.set_pos(pos);
    }

    fn is_colliding_with(&self, pos: Vec2, radius: f32) -> bool {
        ((self.get_pos().x - pos.x).abs() < (self.get_size() + radius))
            && ((self.get_pos().y - pos.y).abs() < (self.get_size() + radius))
    }
}
