use core::f32;

use glam::{Vec2, Vec3, usize};
use rand::random_range;

use super::collision::Collision;
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BombState {
    Planted,
    #[allow(unused)]
    Sliding(Direction),
    Exploding,
}

#[derive(Default, Debug, Clone)]
pub struct Explosion {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

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

const BOMB_TIMER_DEFAULT: f32 = 3.0;
const BOMB_EXPLOSION_TIME: f32 = 2.0;
const BOMB_EXPLOSION_RADIUS: f32 = 0.4;
pub const BOMB_RADIUS: f32 = 0.5;
const BOMB_SLIDE_RADIUS: f32 = 0.45;
const BOMB_SLIDE_SPEED: f32 = 4.0;

const PERCENTAGE_POWERUP_SPAWN: u64 = 15;

impl Bomb {
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

    fn update_model_pos(&mut self) {
        self.objects[0].transform.translation.x = self.position.x;
        self.objects[0].transform.translation.z = self.position.y;
    }

    fn enable_collision(&mut self, players: &[Player]) {
        if self.collision_enabled {
            return;
        }
        if !players[self.owner_id as usize].is_colliding_with(self.position, 0.5) {
            self.collision_enabled = true;
        }
    }

    fn center(&mut self) {
        self.position.x = self.position.x as usize as f32 + 0.5;
        self.position.y = self.position.y as usize as f32 + 0.5;
    }

    fn find_wall(
        &self,
        map: &mut Map,
        dirvec: Vec2,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
    ) -> u8 {
        for i in 1..=self.power {
            let pos = self.position + dirvec * i as f32;
            let elem = map.get_elem_pos(pos);
            match elem {
                MapElement::Empty => continue,
                MapElement::Breakable(_) => {
                    let _ = map.set_elem_pos(pos, MapElement::Empty);
                    if random_range(1..=100) <= PERCENTAGE_POWERUP_SPAWN {
                        power_ups.push(PowerUp::new(pos.y as usize, pos.x as usize, resources));
                    }
                }
                MapElement::Unbreakable(_) => (),
            }
            return i - 1;
        }
        self.power
    }

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

    fn explode(
        &mut self,
        map: &mut Map,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) {
        self.state = BombState::Exploding;
        self.center();

        audio_manager.play_sound_effect(SoundEffect::BombExplosion);

        self.explosion.up = self.find_wall(map, Vec2 { x: 0.0, y: -1.0 }, power_ups, resources);
        self.explosion.down = self.find_wall(map, Vec2 { x: 0.0, y: 1.0 }, power_ups, resources);
        self.explosion.left = self.find_wall(map, Vec2 { x: -1.0, y: 0.0 }, power_ups, resources);
        self.explosion.right = self.find_wall(map, Vec2 { x: 1.0, y: 0.0 }, power_ups, resources);
        self.set_explosion_objects(resources);
    }

    /// tick for bombs that are Planted or Sliding
    fn live_bomb(
        &mut self,
        delta: f32,
        map: &mut Map,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) {
        if (self.timer == 0.0) || (delta >= self.timer) {
            self.timer = 0.0;
            self.explode(map, power_ups, resources, audio_manager);
            return;
        }
        self.timer -= delta;
    }

    /// returns y, x as usize
    fn pos_as_usize(&self) -> (usize, usize) {
        (self.position.y as usize, self.position.x as usize)
    }

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

    /// finds every live bomb near it and explodes it
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

    fn exploding_bomb(
        &mut self,
        delta: f32,
        players: &mut Vec<Player>,
        audio_manager: &mut AudioManager,
    ) {
        if self.timer >= BOMB_EXPLOSION_TIME {
            self.despawn = true;
            players[self.owner_id as usize].bombs_remaining += 1;
        }
        self.timer += delta;

        // kill players
        for player in players.alive() {
            let mut kill = false;

            let (px, py) = player.position.into();
            let (bx, by) = self.position.into();

            if ((px - bx).abs() < (BOMB_EXPLOSION_RADIUS + player.get_size()))
                && ((py + player.get_size())
                    > (by - self.explosion.up as f32 - BOMB_EXPLOSION_RADIUS))
                && ((py - player.get_size())
                    < (by + self.explosion.down as f32 + BOMB_EXPLOSION_RADIUS))
            {
                kill = true;
            }

            if ((py - by).abs() < (BOMB_EXPLOSION_RADIUS + player.get_size()))
                && ((px + player.get_size())
                    > (bx - self.explosion.left as f32 - BOMB_EXPLOSION_RADIUS))
                && ((px - player.get_size())
                    < (bx + self.explosion.right as f32 + BOMB_EXPLOSION_RADIUS))
            {
                kill = true;
            }

            if kill {
                audio_manager.play_sound_effect(SoundEffect::PlayerDeath);
                player.kill();
            }
        }
    }

    fn stop_slide(&mut self, map: &Map) {
        self.bound(map);
        self.center();
        self.update_model_pos();
        self.state = BombState::Planted;
    }

    fn slide(&mut self, direction: Direction, delta: f32, map: &Map, players: &[Player]) {
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
                if self.resolve_collision_with(player.position, player.get_size(), direction) {
                    return self.stop_slide(map);
                }
            }

            motion -= dist;
        }
        self.update_model_pos();
    }

    pub fn tick(
        &mut self,
        delta: f32,
        players: &mut Vec<Player>,
        map: &mut Map,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) {
        match self.state {
            BombState::Planted => self.live_bomb(delta, map, power_ups, resources, audio_manager),
            BombState::Sliding(direction) => {
                self.slide(direction, delta, map, players);
                self.live_bomb(delta, map, power_ups, resources, audio_manager);
            }
            BombState::Exploding => self.exploding_bomb(delta, players, audio_manager),
        }
        self.enable_collision(players);
    }
}

impl Collision for Bomb {
    fn get_pos(&self) -> Vec2 {
        self.position
    }
    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos
    }
    fn get_size(&self) -> f32 {
        match self.state {
            BombState::Exploding => 0.0,
            BombState::Planted => BOMB_RADIUS,
            BombState::Sliding(_) => BOMB_SLIDE_RADIUS,
        }
    }
}
