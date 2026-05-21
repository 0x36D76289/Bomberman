use core::f32;

use glam::{Vec2, Vec3};
use rand::random_range;

use super::{collision::Collision, enemy::Enemy};
use crate::{
    audio::{AudioManager, SoundEffect},
    game::{
        direction::Direction,
        map::{map::Map, map_element::MapElement},
        player::{Alive, Player},
        powerup::PowerUp,
        resources::{ResourceName, Resources},
    },
    graphics::{object::Object, transform::Transform},
};

/// The different states a bomb can be in
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BombState {
    /// A [Planted] bomb is static and ticking
    Planted,
    /// A [Sliding] bomb moves and ticks until it collides or explodes
    Sliding(Direction),
    /// An Exploding bomb kills any player around it and triggers other bombs
    Exploding,
}

/// Events emitted by a [Bomb] tick for scoring and game logic
#[derive(Default, Debug, Clone, Copy)]
pub struct BombEvents {
    pub breakables_destroyed: u32,
    pub enemies_killed: u32,
    pub players_killed: u32,
}

/// When an bomb is in its [Exploding](BombState::Exploding) state it will act in the range of the [Explosion]
#[derive(Default, Debug, Clone)]
pub struct Explosion {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

/// The [Bomb] object, it is spawnable by [Player]
#[derive(Debug, Clone)]
pub struct Bomb {
    pub position: Vec2,
    pub timer: f32,
    pub power: u8,
    pub owner_id: u32,
    pub state: BombState,
    pub collision_enabled: bool,
    pub explosion: Explosion,
    pub despawn: bool,
    pub objects: Vec<Object>,
}

/// How long a [Bomb] ticks before exploding
const BOMB_TIMER_DEFAULT: f32 = 3.0;
/// How long a [Bomb] stays in its [Exploding](BombState::Exploding) before despawning
const BOMB_EXPLOSION_TIME: f32 = 2.0;
/// The radius of a [Bomb] of explosion size 0
const BOMB_EXPLOSION_RADIUS: f32 = 0.4;
/// The radius of a [Bomb] while it is sliding, used for collision detection
const BOMB_SLIDE_RADIUS: f32 = 0.45;
/// How fast a [Bomb] slides in tiles/second
const BOMB_SLIDE_SPEED: f32 = 4.0;

/// The radius of a bomb while it is [Planted](BombState::Planted)
pub const BOMB_RADIUS: f32 = 0.5;

/// The chance of a powerup spawning when a [Bomb] destroys a  [Breakable](MapElement::Breakable) tile
const PERCENTAGE_POWERUP_SPAWN: u64 = 15;

impl Bomb {
    /// The Bomb constructor, used by [Player]s when pressing the Bomb bind
    pub fn new(owner: u32, x: usize, y: usize, power: u8, resources: &Resources) -> Self {
        Self {
            position: Vec2 {
                x: x as f32 + 0.5,
                y: y as f32 + 0.5,
            },
            timer: BOMB_TIMER_DEFAULT,
            power,
            owner_id: owner,
            state: BombState::Planted,
            collision_enabled: false,
            explosion: Explosion::default(),
            despawn: false,
            objects: vec![Self::create_object(x, y, resources)],
        }
    }

    /// The creation of the 3d model at bomb creation
    fn create_object(x: usize, y: usize, resources: &Resources) -> Object {
        Object {
            model: resources.models[&ResourceName::Bomb].clone(),
            texture: Some(resources.textures_index[&ResourceName::Bomb]),
            color: Vec3::ONE,
            transform: Transform {
                translation: Vec3::new(x as f32 + 0.5, 0.0, y as f32 + 0.5),
                scale: Vec3::splat(0.5),
                rotation: Vec3::ZERO,
            },
        }
    }

    /// updates the model's position, used when sliding
    fn update_model_pos(&mut self) {
        self.objects[0].transform.translation.x = self.position.x;
        self.objects[0].transform.translation.z = self.position.y;
    }

    /// When a [Player] spawns a [Bomb] they are unable to collide with one another
    /// As soon as the [Player] stops touching the [Bomb] collision is set to normal
    fn enable_collision(&mut self, players: &[Player]) {
        if self.collision_enabled {
            return;
        }
        if let Some(owner) = players.get(self.owner_id as usize) {
            if !owner.is_colliding_with(self.position, 0.5) {
                self.collision_enabled = true;
            }
        } else {
            self.collision_enabled = true;
        }
    }

    /// Resets a [Bomb]'s position to be aligned with the grid
    fn center(&mut self) {
        self.position.x = self.position.x as usize as f32 + 0.5;
        self.position.y = self.position.y as usize as f32 + 0.5;
    }

    /// Used during explosions to set the explosion range and break nearby
    /// [Breakable](MapElement::Breakable) blocks
    fn find_wall(
        &self,
        map: &mut Map,
        dirvec: Vec2,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
    ) -> (u8, u32) {
        let mut breakables_destroyed = 0;
        for i in 1..=self.power {
            let pos = self.position + dirvec * i as f32;
            let elem = map.get_elem_pos(pos);
            match elem {
                MapElement::Empty | MapElement::Exit(_) => continue,
                MapElement::Breakable(_) => {
                    let _ = map.set_elem_pos(pos, MapElement::Empty);
                    if random_range(1..=100) <= PERCENTAGE_POWERUP_SPAWN {
                        power_ups.push(PowerUp::new(pos.y as usize, pos.x as usize, resources));
                    }
                    breakables_destroyed += 1;
                    return (i - 1, breakables_destroyed);
                }
                MapElement::Unbreakable(_) => {
                    return (i - 1, breakables_destroyed);
                }
            }
        }
        (self.power, breakables_destroyed)
    }

    /// Creates all the other models for the explosion
    fn set_explosion_objects(&mut self, resources: &Resources) {
        self.objects.clear();
        for y in -(self.explosion.up as i16)..=(self.explosion.down as i16) {
            self.objects.push(Self::create_object(
                self.position.x as usize,
                (self.position.y as i16 + y) as usize,
                resources,
            ));
        }
        for x in -(self.explosion.left as i16)..=(self.explosion.right as i16) {
            self.objects.push(Self::create_object(
                (self.position.x as i16 + x) as usize,
                self.position.y as usize,
                resources,
            ));
        }
    }

    /// Triggered when a [Bomb] is set to the [Exploding](BombState::Exploding) state
    fn explode(
        &mut self,
        map: &mut Map,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> u32 {
        self.state = BombState::Exploding;
        self.center();

        audio_manager.play_sound_effect(SoundEffect::BombExplosion);

        let mut breakables_destroyed = 0;

        let (up, up_break) = self.find_wall(map, Vec2 { x: 0.0, y: -1.0 }, power_ups, resources);
        self.explosion.up = up;
        breakables_destroyed += up_break;

        let (down, down_break) = self.find_wall(map, Vec2 { x: 0.0, y: 1.0 }, power_ups, resources);
        self.explosion.down = down;
        breakables_destroyed += down_break;

        let (left, left_break) =
            self.find_wall(map, Vec2 { x: -1.0, y: 0.0 }, power_ups, resources);
        self.explosion.left = left;
        breakables_destroyed += left_break;

        let (right, right_break) =
            self.find_wall(map, Vec2 { x: 1.0, y: 0.0 }, power_ups, resources);
        self.explosion.right = right;
        breakables_destroyed += right_break;
        self.set_explosion_objects(resources);
        breakables_destroyed
    }

    /// Tick for [Bomb]s that are Planted or Sliding
    fn live_bomb(
        &mut self,
        delta: f32,
        map: &mut Map,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> BombEvents {
        if (self.timer == 0.0) || (delta >= self.timer) {
            self.timer = 0.0;
            let breakables_destroyed = self.explode(map, power_ups, resources, audio_manager);
            return BombEvents {
                breakables_destroyed,
                ..Default::default()
            };
        }
        self.timer -= delta;
        BombEvents::default()
    }

    /// returns y, x as usize
    fn pos_as_usize(&self) -> (usize, usize) {
        (self.position.y as usize, self.position.x as usize)
    }

    /// Detects other bombs within explosion radius to trigger chain reactions
    fn in_range(&self, bomb: &Self) -> bool {
        let (self_y, self_x) = self.pos_as_usize();
        let (other_y, other_x) = bomb.pos_as_usize();

        if (self_y == other_y)
            && (other_x >= (self_x - self.explosion.left as usize))
            && (other_x <= (self_x + self.explosion.right as usize))
        {
            return true;
        }

        if (self_x == other_x)
            && (other_y >= (self_y - self.explosion.up as usize))
            && (other_y <= (self_y + self.explosion.down as usize))
        {
            return true;
        }

        false
    }

    /// Finds every live bomb near it and explodes it
    pub fn chain_react(&self, bombs: &mut Vec<Self>) {
        for bomb in bombs {
            if self.in_range(bomb) {
                match bomb.state {
                    BombState::Planted => {
                        bomb.timer = 0.0;
                    }
                    BombState::Sliding(_) => {
                        bomb.state = BombState::Planted;
                        bomb.timer = 0.0;
                    }
                    BombState::Exploding => (),
                }
            }
        }
    }

    /// Checks if the position of a game object is within a [Bomb]'s explosion radius
    fn is_pos_in_explosion(&self, pos: Vec2, radius: f32) -> bool {
        let (px, py) = pos.into();
        let (bx, by) = self.position.into();

        if ((px - bx).abs() < (BOMB_EXPLOSION_RADIUS + radius))
            && ((py + radius) > (by - self.explosion.up as f32 - BOMB_EXPLOSION_RADIUS))
            && ((py - radius) < (by + self.explosion.down as f32 + BOMB_EXPLOSION_RADIUS))
        {
            return true;
        }

        if ((py - by).abs() < (BOMB_EXPLOSION_RADIUS + radius))
            && ((px + radius) > (bx - self.explosion.left as f32 - BOMB_EXPLOSION_RADIUS))
            && ((px - radius) < (bx + self.explosion.right as f32 + BOMB_EXPLOSION_RADIUS))
        {
            return true;
        }

        false
    }

    /// The tick function of a [Bomb] in the [Exploding](BombState::Exploding) State
    fn exploding_bomb(
        &mut self,
        delta: f32,
        players: &mut Vec<Player>,
        enemies: &mut Vec<Enemy>,
        audio_manager: &mut AudioManager,
    ) -> BombEvents {
        let mut events = BombEvents::default();
        if self.timer >= BOMB_EXPLOSION_TIME {
            self.despawn = true;
            if let Some(player) = players.get_mut(self.owner_id as usize) {
                player.bombs_remaining += 1;
            }
        }
        self.timer += delta;

        // kill players
        for player in players.alive() {
            if self.is_pos_in_explosion(player.position, player.get_size()) {
                audio_manager.play_sound_effect(SoundEffect::PlayerDeath);
                player.kill();
                events.players_killed += 1;
            }
        }

        // kill enemies
        for enemy in enemies.iter_mut().filter(|e| e.alive) {
            if self.is_pos_in_explosion(enemy.position, enemy.get_size()) {
                audio_manager.play_sound_effect(SoundEffect::EnemyDeath);
                enemy.kill();
                events.enemies_killed += 1;
            }
        }
        events
    }

    /// Stops a [Sliding](BombExplosion::Sliding) [Bomb] and Plants it
    fn stop_slide(&mut self, map: &Map) {
        self.bound(map);
        self.center();
        self.update_model_pos();
        self.state = BombState::Planted;
    }

    /// The tick function of a [Bomb] in the [Sliding](BombExplosion::Sliding) State
    fn slide(
        &mut self,
        direction: Direction,
        delta: f32,
        map: &Map,
        players: &[Player],
        bombs_pos: &[Vec2],
    ) {
        let mut motion = direction.to_vec2() * delta * BOMB_SLIDE_SPEED;

        let mut dist: Vec2;
        while motion.x != 0.0 || motion.y != 0.0 {
            dist = motion;
            if motion.x.abs() > 1.0 || motion.y.abs() > 1.0 {
                dist = motion.normalize()
            }
            self.position += dist;
            self.bound(map);
            if self.collide_map(map, direction) {
                return self.stop_slide(map);
            }
            for player in players {
                if !player.alive {
                    continue;
                }
                if self.resolve_collision_with(player.position, player.get_size(), direction) {
                    return self.stop_slide(map);
                }
            }
            for pos in bombs_pos {
                if self.resolve_collision_with(*pos, BOMB_RADIUS, direction) {
                    return self.stop_slide(map);
                }
            }

            motion -= dist;
        }
        self.update_model_pos();
    }

    /// The general [Bomb] tick function
    pub fn tick(
        &mut self,
        delta: f32,
        players: &mut Vec<Player>,
        enemies: &mut Vec<Enemy>,
        map: &mut Map,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
        bombs_pos: &[Vec2],
    ) -> BombEvents {
        let events = match self.state {
            BombState::Planted => self.live_bomb(delta, map, power_ups, resources, audio_manager),
            BombState::Sliding(direction) => {
                self.slide(direction, delta, map, players, bombs_pos);
                self.live_bomb(delta, map, power_ups, resources, audio_manager)
            }
            BombState::Exploding => self.exploding_bomb(delta, players, enemies, audio_manager),
        };
        self.enable_collision(players);
        events
    }
}

impl Collision for Bomb {
    fn get_pos(&self) -> Vec2 {
        self.position
    }
    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos
    }
    /// A [Bomb's] size changes depending on its state
    fn get_size(&self) -> f32 {
        match self.state {
            BombState::Exploding => 0.0,
            BombState::Planted => BOMB_RADIUS,
            BombState::Sliding(_) => BOMB_SLIDE_RADIUS,
        }
    }
}
