use crate::game::{
    bomb::Bomb,
    input::Input,
    map::{Map, MapElement},
};

use super::direction::Direction;
use glam::Vec2;

const PLAYER_RADIUS: f32 = 0.4;

pub struct Player {
    pub id: u32,
    pub position: Vec2,
    pub direction: Direction,
    pub alive: bool,
    pub power_level: u32,
    pub speed: f32,
    pub bombs_remaining: u32,
    pub is_human: bool,
    pub can_kick_bomb: bool,
}

impl Player {
    pub fn new(id: u32, position: Vec2, direction: Direction) -> Self {
        Player {
            id: id,
            position: position,
            direction: direction,
            alive: true,
            power_level: 1,
            speed: 1.0,
            bombs_remaining: 1,
            is_human: true,
            can_kick_bomb: false,
        }
    }

    fn bound(&mut self, width: usize, height: usize) {
        self.position.x = self.position.x.max(PLAYER_RADIUS);
        self.position.x = self.position.x.min(width as f32 - PLAYER_RADIUS);
        self.position.y = self.position.y.max(PLAYER_RADIUS);
        self.position.y = self.position.y.min(height as f32 - PLAYER_RADIUS);
    }

    fn resolve_collision(&mut self, pos: Vec2, radius: f32, direction: Direction) {
        if ((self.position.x - pos.x).abs() >= (PLAYER_RADIUS + radius))
            || ((self.position.y - pos.y).abs() >= (PLAYER_RADIUS + radius))
        {
            return;
        }
        match direction {
            Direction::Up => self.position.y = pos.y + radius + PLAYER_RADIUS,
            Direction::Down => self.position.y = pos.y - radius - PLAYER_RADIUS,
            Direction::Left => self.position.x = pos.x + radius + PLAYER_RADIUS,
            Direction::Right => self.position.x = pos.x - radius - PLAYER_RADIUS,
        }
    }

    fn does_map_collide(map: &Map, x: f32, y: f32) -> bool {
        if x < 0.0 || y < 0.0 {
            return false;
        }
        if (x as usize >= map.width) || (y as usize >= map.height) {
            return false;
        }
        return map.content[y as usize * map.width + x as usize] != MapElement::Empty;
    }
    fn collide_map(&mut self, map: &Map, direction: Direction) {
        for y in -1..2 {
            for x in -1..2 {
                if Self::does_map_collide(
                    map,
                    self.position.x + x as f32,
                    self.position.y + y as f32,
                ) {
                    self.resolve_collision(
                        Vec2 {
                            x: (self.position.x + x as f32) as usize as f32 + 0.5,
                            y: (self.position.y + y as f32) as usize as f32 + 0.5,
                        },
                        0.5,
                        direction,
                    );
                }
            }
        }
    }

    fn handle_collisions(&mut self, map: &Map, direction: Direction) {
        self.bound(map.width, map.height);
        self.collide_map(map, direction);
        //TODO:
        //collide players
        //collide bombs
        // check if bomb is ours and has collision disabled
        //collide powerups
    }

    pub fn create_bomb(&mut self) -> Option<Bomb> {
        if self.bombs_remaining == 0 {
            return None;
        }
        //check position
        //check position is bomb
        self.bombs_remaining -= 1;
        return Some(Bomb::new(
            self.id,
            self.position.x as usize,
            self.position.y as usize,
        ));
    }

    pub fn player_move(&mut self, input: Input, delta: f32, map: &Map, bombs: &Vec<Bomb>) {
        let mut motion = input.to_vec2();
        motion *= delta;

        let mut direction: Direction;
        let mut dist: f32;
        while motion.x != 0.0 || motion.y != 0.0 {
            // X tick
            if motion.x != 0.0 {
                if motion.x > 0.0 {
                    direction = Direction::Right;
                    dist = motion.x.abs().min(1.0);
                    motion.x -= dist;
                    self.position.x += dist;
                } else {
                    direction = Direction::Left;
                    dist = motion.x.abs().min(1.0);
                    motion.x += dist;
                    self.position.x -= dist;
                }
                self.handle_collisions(map, direction);
            }
            if motion.y != 0.0 {
                if motion.y > 0.0 {
                    direction = Direction::Down;
                    dist = motion.y.abs().min(1.0);
                    motion.y -= dist;
                    self.position.y += dist;
                    // self.bound(map.width, map.height);
                    // self.collide_down_map(map);
                } else {
                    direction = Direction::Up;
                    dist = motion.y.abs().min(1.0);
                    motion.y += dist;
                    self.position.y -= dist;
                    // self.bound(map.width, map.height);
                    // self.collide_up_map(map);
                }
                self.bound(map.width, map.height);
                self.handle_collisions(map, direction);
            }
        }
    }
}
