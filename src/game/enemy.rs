use crate::graphics::{object::Object, transform::Transform};
use glam::{Vec2, Vec3};
use rand::prelude::*;
use std::collections::VecDeque;

use super::{
    collision::Collision,
    direction::Direction,
    map::map::Map,
    resources::{ResourceName, Resources},
};

const ENEMY_RADIUS: f32 = 0.4;
const ENEMY_SPEED: f32 = 1.5;

/// The different behavior profiles for enemies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyBehavior {
    Aggressive,
    Coward,
    Wander,
}

/// The Enemy is the main obstacle of the singleplayer campaign
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Enemy {
    /// The unique id of an [Enemy], it is also its position in the enemies vector
    pub id: u32,
    /// The position of the [Enemy]
    pub position: Vec2,
    /// The [Enemy] always walks in the direction it faces
    pub direction: Direction,
    /// The behavior profile driving this [Enemy]'s decisions
    behavior: EnemyBehavior,
    /// The speed multiplier applied to the base enemy speed
    speed_multiplier: f32,
    /// Time until the next decision is allowed
    decision_timer: f32,
    /// The base interval between decisions
    decision_interval: f32,
    /// Wether the [Enemy] is alive or not, necessary to preserve the [id](Enemy::id)
    pub alive: bool,
    /// The [Enemy]'s 3d model
    pub object: Option<Object>,
}

impl Enemy {
    /// The [Enemy]'s constructor
    pub fn new(
        id: u32,
        position: Vec2,
        behavior: EnemyBehavior,
        speed_multiplier: f32,
        decision_interval: f32,
        resources: &Resources,
    ) -> Self {
        Self {
            id,
            position,
            direction: Direction::Down,
            behavior,
            speed_multiplier: speed_multiplier.clamp(0.5, 2.5),
            decision_timer: 0.0,
            decision_interval: decision_interval.clamp(0.2, 2.0),
            alive: true,
            object: Some(Self::create_object(
                resources,
                position,
                Direction::Down,
                behavior,
            )),
        }
    }

    /// Creates the 3d model for the [Enemy]
    fn create_object(
        resources: &Resources,
        position: Vec2,
        direction: Direction,
        behavior: EnemyBehavior,
    ) -> Object {
        let dir_vec = direction.to_vec2();
        Object {
            model: resources.models[&ResourceName::Player].clone(), // TODO: Using player model for now
            texture: Some(resources.textures_index[&ResourceName::Player]),
            color: Self::behavior_color(behavior),
            transform: Transform {
                translation: Vec3::new(position.x, 0.0, position.y),
                scale: Vec3::splat(0.35),
                rotation: Vec3::new(0.0, dir_vec.x.atan2(dir_vec.y), 0.0),
            },
        }
    }

    fn behavior_color(behavior: EnemyBehavior) -> Vec3 {
        match behavior {
            EnemyBehavior::Aggressive => Vec3::new(1.0, 0.2, 0.2),
            EnemyBehavior::Coward => Vec3::new(0.2, 0.5, 1.0),
            EnemyBehavior::Wander => Vec3::new(1.0, 0.9, 0.2),
        }
    }

    /// Disables the [Enemy], effectively removing it from the game
    pub fn kill(&mut self) {
        self.alive = false;
        self.object = None;
    }

    fn decide_direction(
        &mut self,
        map: &Map,
        player_pos: Option<Vec2>,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
    ) {
        self.direction = self.pick_direction(map, player_pos, bombs, other_enemies);
        self.decision_timer = self.decision_interval;
    }

    fn pick_direction(
        &self,
        map: &Map,
        player_pos: Option<Vec2>,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
    ) -> Direction {
        match self.behavior {
            EnemyBehavior::Aggressive => {
                if let Some(target) = player_pos {
                    if let Some(dir) =
                        self.bfs_direction_to_target(map, target, bombs, other_enemies)
                    {
                        return dir;
                    }
                    if let Some(dir) =
                        self.greedy_direction(map, target, bombs, other_enemies, false)
                    {
                        return dir;
                    }
                }
                self.wander_direction(map, bombs, other_enemies)
            }
            EnemyBehavior::Coward => {
                if let Some(target) = player_pos {
                    if let Some(dir) = self.flee_direction(map, target, bombs, other_enemies) {
                        return dir;
                    }
                }
                self.wander_direction(map, bombs, other_enemies)
            }
            EnemyBehavior::Wander => self.wander_direction(map, bombs, other_enemies),
        }
    }

    fn wander_direction(
        &self,
        map: &Map,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
    ) -> Direction {
        let mut rng = rand::rng();
        let candidates = self.candidate_directions(map, bombs, other_enemies, false);
        if candidates.is_empty() {
            let directions: Vec<_> = Direction::iterator().collect();
            return **directions.choose(&mut rng).unwrap();
        }
        *candidates.choose(&mut rng).unwrap()
    }

    fn flee_direction(
        &self,
        map: &Map,
        target: Vec2,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
    ) -> Option<Direction> {
        let candidates = self.candidate_directions(map, bombs, other_enemies, true);
        candidates.into_iter().max_by(|a, b| {
            let a_pos = self.next_tile_center(*a);
            let b_pos = self.next_tile_center(*b);
            let a_dist = a_pos.distance_squared(target);
            let b_dist = b_pos.distance_squared(target);
            a_dist
                .partial_cmp(&b_dist)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn greedy_direction(
        &self,
        map: &Map,
        target: Vec2,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
        avoid_danger: bool,
    ) -> Option<Direction> {
        let candidates = self.candidate_directions(map, bombs, other_enemies, avoid_danger);
        candidates.into_iter().min_by(|a, b| {
            let a_pos = self.next_tile_center(*a);
            let b_pos = self.next_tile_center(*b);
            let a_dist = a_pos.distance_squared(target);
            let b_dist = b_pos.distance_squared(target);
            a_dist
                .partial_cmp(&b_dist)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn candidate_directions(
        &self,
        map: &Map,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
        avoid_danger: bool,
    ) -> Vec<Direction> {
        let (tile_x, tile_y) = Self::to_tile(self.position);
        let mut ret = Vec::new();
        for dir in Direction::iterator() {
            let dir_vec = dir.to_vec2();
            let next_x = tile_x + dir_vec.x as i32;
            let next_y = tile_y + dir_vec.y as i32;
            if !map.is_walkable_tile(next_x, next_y) {
                continue;
            }
            let next_pos = Self::tile_center(next_x, next_y);
            if Self::tile_blocked(next_pos, bombs, other_enemies) {
                continue;
            }
            if avoid_danger && Self::tile_dangerous(next_pos, bombs) {
                continue;
            }
            ret.push(*dir);
        }
        ret
    }

    fn tile_blocked(pos: Vec2, bombs: &[super::bomb::Bomb], other_enemies: &[Enemy]) -> bool {
        let occupied_radius_sq = 0.3 * 0.3;
        for bomb in bombs {
            if bomb.position.distance_squared(pos) <= occupied_radius_sq {
                return true;
            }
        }
        for enemy in other_enemies.iter().filter(|e| e.alive) {
            if enemy.position.distance_squared(pos) <= occupied_radius_sq {
                return true;
            }
        }
        false
    }

    fn tile_dangerous(pos: Vec2, bombs: &[super::bomb::Bomb]) -> bool {
        let danger_radius_sq = 1.2 * 1.2;
        bombs
            .iter()
            .any(|bomb| bomb.position.distance_squared(pos) <= danger_radius_sq)
    }

    fn bfs_direction_to_target(
        &self,
        map: &Map,
        target: Vec2,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
    ) -> Option<Direction> {
        let (start_x, start_y) = Self::to_tile(self.position);
        let (target_x, target_y) = Self::to_tile(target);
        if start_x == target_x && start_y == target_y {
            return None;
        }

        let width = map.width as i32;
        let height = map.height as i32;
        if start_x < 0
            || start_y < 0
            || target_x < 0
            || target_y < 0
            || start_x >= width
            || start_y >= height
            || target_x >= width
            || target_y >= height
        {
            return None;
        }

        let mut queue = VecDeque::new();
        let mut visited = vec![false; (width * height) as usize];
        let mut parent: Vec<Option<(i32, i32)>> = vec![None; (width * height) as usize];

        let start_idx = Self::tile_index(start_x, start_y, width);
        let target_idx = Self::tile_index(target_x, target_y, width);
        visited[start_idx] = true;
        queue.push_back((start_x, start_y));

        while let Some((x, y)) = queue.pop_front() {
            if x == target_x && y == target_y {
                break;
            }

            for dir in Direction::iterator() {
                let dir_vec = dir.to_vec2();
                let nx = x + dir_vec.x as i32;
                let ny = y + dir_vec.y as i32;
                if !map.is_walkable_tile(nx, ny) {
                    continue;
                }
                let next_pos = Self::tile_center(nx, ny);
                if Self::tile_blocked(next_pos, bombs, other_enemies) {
                    continue;
                }
                let next_idx = Self::tile_index(nx, ny, width);
                if visited[next_idx] {
                    continue;
                }
                visited[next_idx] = true;
                parent[next_idx] = Some((x, y));
                queue.push_back((nx, ny));
            }
        }

        if !visited[target_idx] {
            return None;
        }

        let mut current = (target_x, target_y);
        let mut prev = parent[target_idx]?;
        while prev != (start_x, start_y) {
            current = prev;
            prev = parent[Self::tile_index(prev.0, prev.1, width)]?;
        }

        let next_pos = Self::tile_center(current.0, current.1);
        Some(Direction::get_direction(&self.position, &next_pos))
    }

    fn to_tile(pos: Vec2) -> (i32, i32) {
        (pos.x.floor() as i32, pos.y.floor() as i32)
    }

    fn tile_center(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32 + 0.5, y as f32 + 0.5)
    }

    fn tile_index(x: i32, y: i32, width: i32) -> usize {
        (y * width + x) as usize
    }

    fn next_tile_center(&self, direction: Direction) -> Vec2 {
        let (tile_x, tile_y) = Self::to_tile(self.position);
        let dir_vec = direction.to_vec2();
        Self::tile_center(tile_x + dir_vec.x as i32, tile_y + dir_vec.y as i32)
    }

    /// The [Enemy]'s tick function runs every tick, simulating all events since last frame
    pub fn tick(
        &mut self,
        delta: f32,
        map: &Map,
        player_pos: Option<Vec2>,
        bombs: &[super::bomb::Bomb],
        other_enemies: &[Enemy],
    ) {
        if !self.alive {
            return;
        }

        self.decision_timer -= delta;
        if self.decision_timer <= 0.0 {
            self.decide_direction(map, player_pos, bombs, other_enemies);
        }

        let motion = self.direction.to_vec2() * delta * ENEMY_SPEED * self.speed_multiplier;
        self.position += motion;

        // Check for map collisions
        if self.collide_map(map, self.direction) {
            self.position -= motion; // step back
            self.decide_direction(map, player_pos, bombs, other_enemies);
        }

        // Check for bomb collisions
        for bomb in bombs {
            if self.is_colliding_with(bomb.get_pos(), bomb.get_size()) {
                self.position -= motion; // step back
                self.decide_direction(map, player_pos, bombs, other_enemies);
                break;
            }
        }

        // Check for collisions with other enemies
        for other_enemy in other_enemies {
            if other_enemy.id != self.id && other_enemy.alive {
                if self.is_colliding_with(other_enemy.get_pos(), other_enemy.get_size()) {
                    self.position -= motion; // step back
                    self.decide_direction(map, player_pos, bombs, other_enemies);
                    break;
                }
            }
        }

        // Update object visuals
        if let Some(obj) = &mut self.object {
            obj.transform.translation = Vec3::new(self.position.x, 0.0, self.position.y);
            let (x, y) = self.direction.to_vec2().into();
            if x != 0.0 || y != 0.0 {
                obj.transform.rotation.y = x.atan2(y);
            }
        }
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
