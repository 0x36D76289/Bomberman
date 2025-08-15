use glam::Vec2;

use crate::game::{
    direction::Direction,
    map::{Map, MapElement},
};

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
        return ((self.get_pos().x - pos.x).abs() < (self.get_size() + radius))
            && ((self.get_pos().y - pos.y).abs() < (self.get_size() + radius));
    }

    fn resolve_collision_with(&mut self, pos: Vec2, radius: f32, direction: Direction) {
        if !self.is_colliding_with(pos, radius) {
            return;
        }
        match direction {
            Direction::Up => self.set_pos(Vec2 {
                x: self.get_pos().x,
                y: pos.y + radius + self.get_size(),
            }),

            Direction::Down => self.set_pos(Vec2 {
                x: self.get_pos().x,
                y: pos.y - radius - self.get_size(),
            }),

            Direction::Left => self.set_pos(Vec2 {
                x: pos.x + radius + self.get_size(),
                y: self.get_pos().y,
            }),

            Direction::Right => self.set_pos(Vec2 {
                x: pos.x - radius - self.get_size(),
                y: self.get_pos().y,
            }),
        }
    }

    fn does_map_collide(map: &Map, x: f32, y: f32) -> bool {
        if x < 0.0 || y < 0.0 {
            return true;
        }
        if (x as usize >= map.width) || (y as usize >= map.height) {
            return true;
        }
        return match map.get_elem(x as usize, y as usize) {
            MapElement::Empty => false,
            MapElement::SpawnPoint => false,
            MapElement::Breakable(_) => true,
            MapElement::Unbreakable(_) => true,
        };
    }

    fn collide_map(&mut self, map: &Map, direction: Direction) {
        for y in -1..2 {
            for x in -1..2 {
                if Self::does_map_collide(
                    map,
                    self.get_pos().x + x as f32,
                    self.get_pos().y + y as f32,
                ) {
                    self.resolve_collision_with(
                        Vec2 {
                            x: (self.get_pos().x + x as f32) as usize as f32 + 0.5,
                            y: (self.get_pos().y + y as f32) as usize as f32 + 0.5,
                        },
                        0.5,
                        direction,
                    );
                }
            }
        }
    }
}
