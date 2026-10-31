use super::direction::Direction;
use crate::{
    game::{
        bomb::{BOMB_TIMER_DEFAULT, Bomb, BombState},
        collision::Collision,
        grid::{Tile, center_of, step, tile_of},
        map::{map::Map, map_element::MapElement},
        movement::walk,
        powerup::Curse,
        resources::{ResourceName, Resources},
    },
    graphics::{object::AnimatedObject, transform::Transform},
    input::{input::Input, input_state::InputState},
};
use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

const BASE_SPEED: f32 = 3.0;
const SPEED_STEP: f32 = 0.4;
pub const MAX_SPEED_LEVEL: u8 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerAnimation {
    Idle,
    Run,
    Bomb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Powers {
    pub bombs: u8,
    pub fire: u8,
    pub speed: u8,
    pub kick: bool,
    pub remote: bool,
    pub wall_pass: bool,
    pub bomb_pass: bool,
}

impl Default for Powers {
    fn default() -> Self {
        Self {
            bombs: 1,
            fire: 2,
            speed: 0,
            kick: false,
            remote: false,
            wall_pass: false,
            bomb_pass: false,
        }
    }
}

impl Powers {
    pub fn lose_specials(&mut self) {
        self.kick = false;
        self.remote = false;
        self.wall_pass = false;
        self.bomb_pass = false;
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub position: Vec2,
    pub direction: Direction,
    pub alive: bool,
    pub powers: Powers,
    pub bombs_remaining: u32,
    pub invincible: f32,
    pub curse: Option<(Curse, f32)>,
    pub base_color: Vec3,
    pub team: Option<u32>,
    pub is_human: bool,
    pub object: Option<AnimatedObject>,
    pub moving: bool,
    horizontal_last: bool,
    animation: PlayerAnimation,
    bomb_animation: f32,
}

impl Player {
    pub fn new(
        id: u32,
        position: Vec2,
        direction: Direction,
        resources: &Resources,
        is_human: bool,
        powers: Powers,
    ) -> Self {
        let position = center_of(tile_of(position));
        Player {
            id,
            position,
            direction,
            alive: true,
            powers,
            bombs_remaining: powers.bombs as u32,
            invincible: 0.0,
            curse: None,
            base_color: Vec3::ONE,
            team: None,
            is_human,
            object: Some(Self::create_object(resources, position, direction)),
            moving: false,
            horizontal_last: false,
            animation: PlayerAnimation::Idle,
            bomb_animation: 0.0,
        }
    }

    fn create_object(
        resources: &Resources,
        position: Vec2,
        direction: Direction,
    ) -> AnimatedObject {
        let mut mesh = resources.player_mesh.clone();
        mesh.animator
            .set_animation(Some(resources.player_animations.idle.clone()));
        mesh.animator.update(0.0, &mut mesh.root_joint);
        AnimatedObject {
            mesh,
            texture: Some(resources.textures_index[&ResourceName::Player]),
            color: Vec3::ONE,
            tint_whites: true,
            transform: Transform {
                translation: Vec3::new(position.x, -1.5, position.y),
                scale: Vec3::splat(0.35),
                rotation: Vec3::new(0.0, direction.yaw(), 0.0),
            },
        }
    }

    pub fn animate(&mut self, resources: &Resources, delta: f32, planted: bool) {
        let animations = &resources.player_animations;
        if planted {
            self.bomb_animation = animations.bomb.length.as_secs_f32();
        }
        self.bomb_animation = (self.bomb_animation - delta).max(0.0);
        let state = if planted || self.bomb_animation > 0.0 {
            PlayerAnimation::Bomb
        } else if self.moving {
            PlayerAnimation::Run
        } else {
            PlayerAnimation::Idle
        };
        let Some(object) = &mut self.object else {
            return;
        };
        let animator = &mut object.mesh.animator;
        if planted || state != self.animation {
            animator.set_animation(None);
            animator.set_animation(Some(
                match state {
                    PlayerAnimation::Bomb => &animations.bomb,
                    PlayerAnimation::Run => &animations.run,
                    PlayerAnimation::Idle => &animations.idle,
                }
                .clone(),
            ));
            self.animation = state;
        }
        animator.update(delta, &mut object.mesh.root_joint);
    }

    pub fn fireproof(&self) -> bool {
        self.invincible > 0.0
    }

    pub fn cursed_with(&self, curse: Curse) -> bool {
        self.curse.is_some_and(|(c, _)| c == curse)
    }

    pub fn tick_status(&mut self, delta: f32) {
        self.invincible = (self.invincible - delta).max(0.0);
        if let Some((_, left)) = &mut self.curse {
            *left -= delta;
        }
        self.curse = self.curse.filter(|(_, left)| *left > 0.0);
        let (timer, color) = match self.curse {
            _ if self.invincible > 0.0 => (self.invincible, Vec3::new(1.8, 1.5, 0.4)),
            Some((_, left)) => (left, Vec3::new(0.6, 0.2, 0.9)),
            None => (0.0, self.base_color),
        };
        if let Some(obj) = &mut self.object {
            obj.color = if timer > 0.0 && ((timer * 8.0) as u32).is_multiple_of(2) {
                color
            } else {
                self.base_color
            };
        }
    }

    pub fn create_bomb(
        &mut self,
        resources: &Resources,
        bombs: &[Bomb],
        map: &Map,
    ) -> Option<Bomb> {
        if self.bombs_remaining == 0 || self.cursed_with(Curse::Constipation) {
            return None;
        }
        let tile = tile_of(self.position);
        if matches!(map.get_elem_pos(self.position), MapElement::Breakable(_)) {
            return None;
        }
        if bombs
            .iter()
            .any(|bomb| bomb.state != BombState::Exploding && tile_of(bomb.position) == tile)
        {
            return None;
        }
        self.bombs_remaining -= 1;
        Some(Bomb::new(
            self.id,
            tile.0 as usize,
            tile.1 as usize,
            if self.cursed_with(Curse::WeakFire) {
                1
            } else {
                self.powers.fire
            },
            if self.cursed_with(Curse::ShortFuse) {
                1.0
            } else {
                BOMB_TIMER_DEFAULT
            },
            self.powers.remote,
            resources,
        ))
    }

    fn wishes(&mut self, input: Input) -> Vec<Direction> {
        let pressed = |state: InputState| state == InputState::Pressed;
        if pressed(input.left()) || pressed(input.right()) {
            self.horizontal_last = true;
        } else if pressed(input.up()) || pressed(input.down()) {
            self.horizontal_last = false;
        }
        let motion = input.as_vec2();
        let horizontal = (motion.x != 0.0).then_some(if motion.x > 0.0 {
            Direction::Right
        } else {
            Direction::Left
        });
        let vertical = (motion.y != 0.0).then_some(if motion.y > 0.0 {
            Direction::Down
        } else {
            Direction::Up
        });
        let mut wishes = if self.horizontal_last {
            [horizontal, vertical]
        } else {
            [vertical, horizontal]
        };
        wishes.sort_by_key(|w| w.is_none());
        wishes.into_iter().flatten().collect()
    }

    pub fn walk_input(
        &mut self,
        input: Input,
        delta: f32,
        walkable: impl Fn(Tile) -> bool,
        bomb_tiles: &[Tile],
    ) -> Option<(Tile, Direction)> {
        self.moving = false;
        if !self.alive {
            return None;
        }
        let wishes = self.wishes(input);
        if wishes.is_empty() {
            return None;
        }
        let start = tile_of(self.position);
        let distance = delta * self.speed();
        let result = walk(&mut self.position, &wishes, distance, |tile| {
            !walkable(tile) || (tile != start && bomb_tiles.contains(&tile))
        });
        self.moving = result.travelled > 0.0;
        if let Some(facing) = result.facing {
            self.direction = facing;
        }
        result
            .bumped
            .filter(|(tile, _)| !self.moving && bomb_tiles.contains(tile))
    }

    pub fn footprint(&self) -> Vec<Tile> {
        let mut tiles = vec![tile_of(self.position)];
        for offset in [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y] {
            let tile = tile_of(self.position + offset * 0.45);
            if !tiles.contains(&tile) {
                tiles.push(tile);
            }
        }
        tiles
    }

    pub fn player_move(
        &mut self,
        input: Input,
        delta: f32,
        map: &Map,
        bombs: &mut [Bomb],
        occupied: &[Tile],
    ) {
        let bomb_tiles: Vec<Tile> = bombs
            .iter()
            .filter(|b| b.state != BombState::Exploding)
            .map(|b| tile_of(b.position))
            .collect();
        let burning: Vec<Tile> = bombs.iter().flat_map(|b| b.burning_tiles()).collect();
        let fireproof = self.fireproof();
        let wall_pass = self.powers.wall_pass;
        if let Some((tile, direction)) = self.walk_input(
            if self.cursed_with(Curse::Reverse) {
                input.reversed()
            } else {
                input
            },
            delta,
            |tile: Tile| {
                (map.is_walkable_tile(tile.0, tile.1)
                    || (wall_pass && map.is_breakable_tile(tile.0, tile.1)))
                    && (fireproof || !burning.contains(&tile))
                    && !occupied.contains(&tile)
            },
            if self.powers.bomb_pass {
                &[]
            } else {
                &bomb_tiles
            },
        ) && self.powers.kick
        {
            let beyond = step(tile, direction);
            if map.is_walkable_tile(beyond.0, beyond.1)
                && !burning.contains(&beyond)
                && !bomb_tiles.contains(&beyond)
                && let Some(bomb) = bombs
                    .iter_mut()
                    .find(|b| b.state == BombState::Planted && tile_of(b.position) == tile)
            {
                bomb.state = BombState::Sliding(direction);
            }
        }

        if let Some(obj) = &mut self.object {
            obj.transform.translation = Vec3::new(
                self.position.x,
                obj.transform.translation.y,
                self.position.y,
            );
            obj.transform.rotation.y = self.direction.yaw();
        }
    }

    pub fn speed(&self) -> f32 {
        match self.curse {
            Some((Curse::Fast, _)) => BASE_SPEED + SPEED_STEP * (MAX_SPEED_LEVEL + 3) as f32,
            Some((Curse::Slow, _)) => BASE_SPEED * 0.5,
            _ => BASE_SPEED + SPEED_STEP * self.powers.speed.min(MAX_SPEED_LEVEL) as f32,
        }
    }

    pub fn respawn(&mut self, resources: &Resources, position: Vec2, invincible: f32) {
        self.position = center_of(tile_of(position));
        self.alive = true;
        self.moving = false;
        self.curse = None;
        self.invincible = invincible;
        self.animation = PlayerAnimation::Idle;
        self.bomb_animation = 0.0;
        self.powers.lose_specials();
        self.object = Some(Self::create_object(
            resources,
            self.position,
            self.direction,
        ));
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.moving = false;
        self.object = None;
    }
}

impl Collision for Player {
    fn get_pos(&self) -> Vec2 {
        self.position
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos;
    }

    fn get_size(&self) -> f32 {
        0.4
    }
}
