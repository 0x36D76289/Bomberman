use super::{
    direction::Direction,
    grid::{Tile, TileGrid, center_of, manhattan, step, tile_of},
    resources::{ResourceName, Resources},
};
use crate::graphics::{object::AnimatedObject, transform::Transform};
use glam::{Vec2, Vec3};
use rand::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Mind {
    chase_range: i32,
    chase_chance: f32,
    turn_chance: f32,
    bounces: bool,
    avoids_blasts: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyKind {
    Balloom,
    Onil,
    Doll,
    Minvo,
    Boss,
}

impl EnemyKind {
    pub fn model(self) -> (&'static str, f32) {
        match self {
            EnemyKind::Balloom => ("character-oobi", 0.7),
            EnemyKind::Onil => ("character-oodi", 0.7),
            EnemyKind::Doll => ("character-ooli", 0.7),
            EnemyKind::Minvo => ("character-oopi", 0.7),
            EnemyKind::Boss => ("character-oozi", 1.3),
        }
    }

    pub fn from_level_char(c: char) -> Option<Self> {
        match c {
            'b' => Some(EnemyKind::Balloom),
            'o' => Some(EnemyKind::Onil),
            'd' => Some(EnemyKind::Doll),
            'm' => Some(EnemyKind::Minvo),
            'B' => Some(EnemyKind::Boss),
            _ => None,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            EnemyKind::Balloom => 1.5,
            EnemyKind::Onil => 2.2,
            EnemyKind::Doll => 2.0,
            EnemyKind::Minvo => 3.0,
            EnemyKind::Boss => 2.0,
        }
    }

    fn mind(self) -> Mind {
        match self {
            EnemyKind::Balloom => Mind {
                chase_range: 0,
                chase_chance: 0.0,
                turn_chance: 0.3,
                bounces: false,
                avoids_blasts: false,
            },
            EnemyKind::Onil => Mind {
                chase_range: 6,
                chase_chance: 0.5,
                turn_chance: 0.4,
                bounces: false,
                avoids_blasts: false,
            },
            EnemyKind::Doll => Mind {
                chase_range: 0,
                chase_chance: 0.0,
                turn_chance: 0.1,
                bounces: true,
                avoids_blasts: false,
            },
            EnemyKind::Minvo => Mind {
                chase_range: 8,
                chase_chance: 0.8,
                turn_chance: 0.4,
                bounces: false,
                avoids_blasts: false,
            },
            EnemyKind::Boss => Mind {
                chase_range: 64,
                chase_chance: 1.0,
                turn_chance: 0.3,
                bounces: false,
                avoids_blasts: true,
            },
        }
    }

    pub fn tint(self) -> Vec3 {
        match self {
            EnemyKind::Balloom => Vec3::new(1.0, 0.6, 0.25),
            EnemyKind::Onil => Vec3::new(0.35, 0.75, 1.0),
            EnemyKind::Doll => Vec3::new(1.0, 0.45, 0.8),
            EnemyKind::Minvo => Vec3::new(0.65, 0.4, 1.0),
            EnemyKind::Boss => Vec3::new(1.0, 0.25, 0.2),
        }
    }

    pub fn health(self) -> u32 {
        match self {
            EnemyKind::Boss => 5,
            _ => 1,
        }
    }

    pub fn score(self) -> u32 {
        match self {
            EnemyKind::Balloom => 100,
            EnemyKind::Onil => 200,
            EnemyKind::Doll => 400,
            EnemyKind::Minvo => 800,
            EnemyKind::Boss => 5000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub position: Vec2,
    pub direction: Direction,
    pub kind: EnemyKind,
    pub health: u32,
    hit_cooldown: f32,
    speed_multiplier: f32,
    awareness: f32,
    target: Tile,
    pub moving: bool,
    pub alive: bool,
    pub object: Option<AnimatedObject>,
    walking: bool,
}

impl Enemy {
    pub fn new(
        position: Vec2,
        kind: EnemyKind,
        speed_multiplier: f32,
        awareness: f32,
        resources: &Resources,
    ) -> Self {
        let tile = tile_of(position);
        let position = center_of(tile);
        let direction = Direction::random();
        Self {
            position,
            direction,
            kind,
            health: kind.health(),
            hit_cooldown: 0.0,
            speed_multiplier: speed_multiplier.clamp(0.5, 2.0),
            awareness: awareness.clamp(0.3, 1.5),
            target: tile,
            moving: false,
            alive: true,
            object: Some(Self::create_object(resources, position, direction, kind)),
            walking: false,
        }
    }

    fn create_object(
        resources: &Resources,
        position: Vec2,
        direction: Direction,
        kind: EnemyKind,
    ) -> AnimatedObject {
        let model = &resources.enemies[&kind];
        let mut mesh = model.mesh.clone();
        mesh.animator.set_animation(Some(model.idle.clone()));
        mesh.animator.update(0.0, &mut mesh.root_joint);
        let scale = kind.model().1 / 86.8;
        AnimatedObject {
            mesh,
            texture: Some(resources.textures_index[&ResourceName::Enemy(kind)]),
            color: kind.tint(),
            tint_whites: false,
            transform: Transform {
                translation: Vec3::new(position.x, 0.0, position.y),
                scale: Vec3::new(scale, -scale, scale),
                rotation: Vec3::new(0.0, direction.yaw(), 0.0),
            },
        }
    }

    pub fn animate(&mut self, resources: &Resources, delta: f32) {
        let walking = self.moving;
        let Some(object) = &mut self.object else {
            return;
        };
        let animator = &mut object.mesh.animator;
        if walking != self.walking {
            let model = &resources.enemies[&self.kind];
            animator.set_animation(Some(
                (if walking { &model.walk } else { &model.idle }).clone(),
            ));
            self.walking = walking;
        }
        animator.update(delta, &mut object.mesh.root_joint);
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.moving = false;
        self.object = None;
    }

    pub fn hit(&mut self) -> Option<bool> {
        if !self.alive || self.hit_cooldown > 0.0 {
            return None;
        }
        self.health = self.health.saturating_sub(1);
        if self.health == 0 {
            self.kill();
            return Some(true);
        }
        self.hit_cooldown = 1.2;
        self.speed_multiplier = (self.speed_multiplier * 1.15).min(2.0);
        Some(false)
    }

    pub fn touches(&self, pos: Vec2) -> bool {
        let gap = (self.position - pos).abs();
        self.alive && gap.x < 0.6 && gap.y < 0.6
    }

    fn choose(
        &self,
        grid: &TileGrid,
        blast: &[f32],
        player_pos: Option<Vec2>,
        occupied: &[Tile],
        rng: &mut impl Rng,
    ) -> Option<Direction> {
        let mind = self.kind.mind();
        let here = self.target;
        let safe =
            |tile: Tile| !mind.avoids_blasts || grid.index(tile).is_none_or(|i| blast[i] > 1.5);
        let free = |tile: Tile| grid.walkable(tile) && !occupied.contains(&tile);
        let mut options: Vec<Direction> = Direction::iterator()
            .copied()
            .filter(|direction| {
                let next = step(here, *direction);
                free(next) && safe(next)
            })
            .collect();
        if options.is_empty() {
            options = Direction::iterator()
                .copied()
                .filter(|d| free(step(here, *d)))
                .collect();
        }
        if options.is_empty() {
            return None;
        }

        if let Some(player) = player_pos
            && mind.chase_range > 0
        {
            let goal = tile_of(player);
            let range = (mind.chase_range as f32 * self.awareness).round() as i32;
            if manhattan(here, goal) <= range
                && rng.random::<f32>() < mind.chase_chance * self.awareness
                && let Some(next) = grid
                    .bfs(here, (range * 2) as u32, |tile, _| safe(tile))
                    .first_step(goal)
                && let Some(direction) = options.iter().find(|d| step(here, **d) == next)
            {
                return Some(*direction);
            }
        }

        let forward = self.direction;
        let back = forward.opposite();
        let turns: Vec<Direction> = options
            .iter()
            .copied()
            .filter(|d| *d != forward && *d != back)
            .collect();
        if options.contains(&forward) {
            if turns.is_empty() || rng.random::<f32>() >= mind.turn_chance {
                return Some(forward);
            }
            return turns.choose(rng).copied();
        }
        if mind.bounces && options.contains(&back) {
            return Some(back);
        }
        turns.choose(rng).copied().or(Some(back))
    }

    pub fn tiles(&self) -> Vec<Tile> {
        let mut tiles = vec![tile_of(self.position), self.target];
        if self.position.distance(center_of(self.target)) > 1e-4 {
            tiles.push(step(self.target, self.direction.opposite()));
        }
        tiles
    }

    pub fn tick(
        &mut self,
        delta: f32,
        grid: &TileGrid,
        blast: &[f32],
        player_pos: Option<Vec2>,
        occupied: &[Tile],
    ) {
        if !self.alive {
            return;
        }

        if self.hit_cooldown > 0.0 {
            self.hit_cooldown -= delta;
            if let Some(obj) = &mut self.object {
                obj.color = if self.hit_cooldown > 0.0 && (self.hit_cooldown * 10.0) as i32 % 2 == 0
                {
                    Vec3::ONE
                } else {
                    self.kind.tint()
                };
            }
        }

        let here = tile_of(self.position);
        if self.target != here && !grid.walkable(self.target) {
            self.target = here;
            self.direction = self.direction.opposite();
        }

        let mut rng = rand::rng();
        let mut budget = self.kind.speed() * self.speed_multiplier * delta;
        self.moving = false;
        for _ in 0..4 {
            let gap = center_of(self.target) - self.position;
            let distance = gap.length();
            if distance > 1e-4 {
                let moved = distance.min(budget);
                self.position += gap / distance * moved;
                budget -= moved;
                self.moving = true;
                if budget <= 1e-4 {
                    break;
                }
            }
            self.position = center_of(self.target);
            match self.choose(grid, blast, player_pos, occupied, &mut rng) {
                Some(direction) => {
                    self.direction = direction;
                    self.target = step(self.target, direction);
                }
                None => break,
            }
        }

        if let Some(obj) = &mut self.object {
            obj.transform.translation = Vec3::new(self.position.x, 0.0, self.position.y);
            obj.transform.rotation.y = self.direction.yaw();
        }
    }
}
