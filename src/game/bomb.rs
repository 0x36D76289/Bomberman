use super::{collision::Collision, enemy::Enemy};
use crate::{
    audio::{AudioManager, SoundEffect},
    game::{
        direction::Direction,
        grid::{Tile, center_of, tile_of, tile_transform},
        map::{map::Map, map_element::MapElement},
        player::Player,
        powerup::{PowerUp, PowerUpType},
        resources::{ResourceName, Resources},
    },
    graphics::{object::Object, transform::Transform},
};
use glam::{Vec2, Vec3};
use rand::random_range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BombState {
    Planted,
    Sliding(Direction),
    Exploding,
}

pub struct BombWorld<'a> {
    pub players: &'a mut [Player],
    pub enemies: &'a mut [Enemy],
    pub map: &'a mut Map,
    pub power_ups: &'a mut Vec<PowerUp>,
    pub resources: &'a Resources,
    pub audio_manager: &'a mut AudioManager,
    pub bombs_pos: &'a [Vec2],
    pub burning: &'a [Tile],
}

#[derive(Default, Debug, Clone, Copy)]
pub struct BombEvents {
    pub breakables_destroyed: u32,
    pub enemies_score: u32,
    pub explosions: u32,
    pub exit_hit: bool,
}

#[derive(Debug, Clone)]
pub struct FlamePiece {
    object: Object,
    distance: u8,
    full_scale: Vec3,
    core: bool,
    center: bool,
}

#[derive(Debug, Clone)]
pub struct BurningBlock {
    pub tile: Tile,
    object: Object,
    full_scale: Vec3,
    color: Vec3,
}

#[derive(Debug, Clone)]
pub struct Bomb {
    pub position: Vec2,
    pub timer: f32,
    pub power: u8,
    pub owner_id: u32,
    pub state: BombState,
    pub collision_enabled: bool,
    pub remote: bool,
    explosion: [u8; 4],
    pub despawn: bool,
    objects: Vec<Object>,
    flames: Vec<FlamePiece>,
    burning: Vec<BurningBlock>,
    hidden: Vec<PowerUp>,
}

pub const BOMB_TIMER_DEFAULT: f32 = 3.0;
pub const BOMB_EXPLOSION_TIME: f32 = 0.6;

const FLAME_GROW: f32 = 0.08;
const FLAME_FADE: f32 = 0.2;
const FLAME_CORE_COLOR: Vec3 = Vec3::new(0.99, 0.99, 0.66);
const RED_HOT_COLOR: Vec3 = Vec3::new(1.6, 0.4, 0.1);
const BOMB_SCALE: f32 = 0.5;

pub fn flame_size(age: f32, distance: u8) -> f32 {
    let local = age - distance as f32 * 0.02;
    if local <= 0.0 || age >= BOMB_EXPLOSION_TIME {
        return 0.0;
    }
    (if local < FLAME_GROW {
        let t = local / FLAME_GROW;
        1.15 * (1.0 - (1.0 - t) * (1.0 - t))
    } else {
        1.0 + 0.15 * (1.0 - (local - FLAME_GROW) / 0.06).max(0.0)
    }) * (1.0
        + 0.08
            * ((age * 12.0).floor() * 2.3 + distance as f32 * 1.7)
                .sin()
                .signum())
        * ((BOMB_EXPLOSION_TIME - age) / FLAME_FADE).clamp(0.0, 1.0)
}

fn pulse_phase(timer: f32) -> f32 {
    let elapsed = (BOMB_TIMER_DEFAULT - timer).max(0.0);
    std::f32::consts::TAU * (2.0 * elapsed + elapsed * elapsed)
}

pub fn pulse(timer: f32) -> f32 {
    1.0 + 0.1 * pulse_phase(timer).sin().signum()
}

impl Bomb {
    pub fn new(
        owner: u32,
        x: usize,
        y: usize,
        power: u8,
        timer: f32,
        remote: bool,
        resources: &Resources,
    ) -> Self {
        Self {
            position: Vec2 {
                x: x as f32 + 0.5,
                y: y as f32 + 0.5,
            },
            timer,
            power,
            owner_id: owner,
            state: BombState::Planted,
            collision_enabled: false,
            remote,
            explosion: [0; 4],
            despawn: false,
            objects: vec![Self::create_object(x, y, resources)],
            flames: Vec::new(),
            burning: Vec::new(),
            hidden: Vec::new(),
        }
    }

    fn create_object(x: usize, y: usize, resources: &Resources) -> Object {
        resources.object(
            ResourceName::Bomb,
            tile_transform(x, y, 0.0, BOMB_SCALE),
            Vec3::ONE,
        )
    }

    fn update_model_pos(&mut self) {
        if let Some(object) = self.objects.first_mut() {
            object.transform.translation.x = self.position.x;
            object.transform.translation.z = self.position.y;
        }
    }

    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        self.objects
            .iter()
            .chain(self.flames.iter().map(|f| &f.object))
            .chain(self.burning.iter().map(|b| &b.object))
    }

    pub fn burning_tiles(&self) -> impl Iterator<Item = Tile> + '_ {
        self.burning.iter().map(|b| b.tile)
    }

    fn enable_collision(&mut self, players: &[Player]) {
        if self.collision_enabled {
            return;
        }
        match players.get(self.owner_id as usize) {
            Some(owner) if owner.alive && tile_of(owner.position) == tile_of(self.position) => (),
            _ => self.collision_enabled = true,
        }
    }

    fn center(&mut self) {
        self.position.x = self.position.x as usize as f32 + 0.5;
        self.position.y = self.position.y as usize as f32 + 0.5;
    }

    fn find_wall(
        &mut self,
        map: &mut Map,
        dirvec: Vec2,
        resources: &Resources,
        burning: &[Tile],
    ) -> (u8, u32) {
        for i in 1..=self.power {
            let pos = self.position + dirvec * i as f32;
            let tile = tile_of(pos);
            if burning.contains(&tile) {
                return (i - 1, 0);
            }
            match map.get_elem_pos(pos) {
                MapElement::Empty | MapElement::Exit(_) => continue,
                MapElement::Breakable(object) => {
                    let object = object.clone();
                    map.set_elem_pos(pos, MapElement::Empty);
                    self.burning.push(BurningBlock {
                        tile,
                        full_scale: object.transform.scale,
                        color: object.color,
                        object,
                    });
                    let cell = (tile.0 as usize, tile.1 as usize);
                    if let Some(kind) =
                        match map.hidden_bonuses.iter().position(|(t, _)| *t == cell) {
                            Some(index) => Some(map.hidden_bonuses.swap_remove(index).1),
                            None if map.random_bonuses
                                && map.exit != Some(cell)
                                && random_range(1..=100u64) <= 20 =>
                            {
                                Some(PowerUpType::random_battle())
                            }
                            None => None,
                        }
                    {
                        self.hidden.push(PowerUp::new(tile, kind, resources));
                    }
                    return (i - 1, 1);
                }
                MapElement::Unbreakable(_) => {
                    return (i - 1, 0);
                }
            }
        }
        (self.power, 0)
    }

    fn set_explosion_objects(&mut self, resources: &Resources) {
        self.objects.clear();
        self.flames.clear();
        let origin = tile_of(self.position);
        let mut piece = |name: ResourceName, tile: Tile, distance: u8, angle: f32, core: bool| {
            let center = name == ResourceName::FlameCenter;
            self.flames.push(FlamePiece {
                object: Object {
                    model: resources.models[&name].clone(),
                    texture: Some(-2),
                    color: if core {
                        FLAME_CORE_COLOR
                    } else {
                        Vec3::new(0.85, 0.16, 0.0)
                    },
                    transform: Transform {
                        translation: Vec3::new(tile.0 as f32 + 0.5, 0.0, tile.1 as f32 + 0.5),
                        scale: Vec3::splat(0.001),
                        rotation: Vec3::new(0.0, angle, 0.0),
                    },
                },
                distance,
                full_scale: match (center, core) {
                    (true, false) => Vec3::new(1.0, 1.05, 1.0),
                    (true, true) => Vec3::new(0.62, 1.4, 0.62),
                    (false, false) => Vec3::new(1.0, 0.9, 0.85),
                    (false, true) => Vec3::new(1.0, 1.25, 0.4),
                },
                core,
                center,
            });
        };
        for core in [false, true] {
            piece(ResourceName::FlameCenter, origin, 0, 0.0, core);
            for angle in [0.0, std::f32::consts::FRAC_PI_2] {
                piece(ResourceName::FlameArm, origin, 0, angle, core);
            }
        }
        for direction in Direction::iterator().copied() {
            let reach = self.explosion[direction as usize];
            let v = direction.to_vec2();
            for distance in 1..=reach {
                for core in [false, true] {
                    piece(
                        if distance == reach {
                            ResourceName::FlameTip
                        } else {
                            ResourceName::FlameArm
                        },
                        (
                            origin.0 + v.x as i32 * distance as i32,
                            origin.1 + v.y as i32 * distance as i32,
                        ),
                        distance,
                        (-v.y).atan2(v.x),
                        core,
                    );
                }
            }
        }
    }

    fn animate_explosion(&mut self) {
        let age = self.timer;
        for flame in &mut self.flames {
            let size = flame_size(age, flame.distance).max(0.001);
            flame.object.transform.scale =
                flame.full_scale * Vec3::new(if flame.center { size } else { 1.0 }, size, size);
            if flame.core {
                flame.object.color = FLAME_CORE_COLOR.lerp(
                    Vec3::new(0.99, 0.6, 0.0),
                    ((age - (BOMB_EXPLOSION_TIME - FLAME_FADE)) / FLAME_FADE).clamp(0.0, 1.0),
                );
            }
        }
        let burnt = (age / BOMB_EXPLOSION_TIME).clamp(0.0, 1.0);
        for block in &mut self.burning {
            block.object.color = if burnt < 0.3 {
                block.color.lerp(RED_HOT_COLOR, burnt / 0.3)
            } else {
                RED_HOT_COLOR.lerp(Vec3::new(0.12, 0.05, 0.02), ((burnt - 0.3) / 0.3).min(1.0))
            };
            let collapse = (1.0 - burnt).max(0.001);
            block.object.transform.scale =
                block.full_scale * Vec3::new(0.7 + 0.3 * collapse, collapse, 0.7 + 0.3 * collapse);
        }
    }

    fn explode(
        &mut self,
        map: &mut Map,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
        burning: &[Tile],
    ) -> BombEvents {
        self.state = BombState::Exploding;
        self.center();

        audio_manager.play_sound_effect(SoundEffect::BombExplosion);

        let mut breakables_destroyed = 0;
        for direction in Direction::iterator().copied() {
            let (reach, broken) = self.find_wall(map, direction.to_vec2(), resources, burning);
            self.explosion[direction as usize] = reach;
            breakables_destroyed += broken;
        }
        if breakables_destroyed > 0 {
            audio_manager.play_sound_effect(SoundEffect::WallBreak);
        }
        power_ups.retain(|p| !self.covers(p.tile));
        self.set_explosion_objects(resources);
        self.animate_explosion();
        BombEvents {
            breakables_destroyed,
            explosions: 1,
            exit_hit: map.exit.is_some_and(|(x, y)| {
                matches!(map.get_elem(x, y), MapElement::Exit(_))
                    && self.covers((x as i32, y as i32))
            }),
            ..Default::default()
        }
    }

    fn live_bomb(
        &mut self,
        delta: f32,
        map: &mut Map,
        power_ups: &mut Vec<PowerUp>,
        resources: &Resources,
        audio_manager: &mut AudioManager,
        burning: &[Tile],
    ) -> BombEvents {
        if (self.timer == 0.0) || (delta >= self.timer) {
            self.timer = 0.0;
            return self.explode(map, power_ups, resources, audio_manager, burning);
        }
        if !self.remote {
            self.timer -= delta;
        }
        if let Some(object) = self.objects.first_mut() {
            object.transform.scale = Vec3::splat(BOMB_SCALE * pulse(self.timer));
        }
        BombEvents::default()
    }

    pub fn chain_react(&self, bombs: &mut [Self]) {
        for bomb in bombs {
            if self.covers(tile_of(bomb.position)) {
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

    pub fn covers(&self, tile: Tile) -> bool {
        let origin = tile_of(self.position);
        let (dx, dy) = (tile.0 - origin.0, tile.1 - origin.1);
        let reach = |direction: Direction| self.explosion[direction as usize] as i32;
        (dy == 0 && dx >= -reach(Direction::Left) && dx <= reach(Direction::Right))
            || (dx == 0 && dy >= -reach(Direction::Up) && dy <= reach(Direction::Down))
    }

    fn exploding_bomb(
        &mut self,
        delta: f32,
        players: &mut [Player],
        enemies: &mut [Enemy],
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
        self.animate_explosion();

        for player in players.iter_mut().filter(|p| p.alive) {
            if !player.fireproof() && self.covers(tile_of(player.position)) {
                audio_manager.play_sound_effect(SoundEffect::PlayerDeath);
                player.kill();
            }
        }

        for enemy in enemies.iter_mut().filter(|e| e.alive) {
            if self.covers(tile_of(enemy.position)) {
                match enemy.hit() {
                    Some(true) => {
                        audio_manager.play_sound_effect(SoundEffect::EnemyDeath);
                        events.enemies_score += enemy.kind.score();
                    }
                    Some(false) => audio_manager.play_sound_effect(SoundEffect::EnemyHit),
                    None => (),
                }
            }
        }
        events
    }

    fn stop_slide(&mut self, map: &Map) {
        self.bound(map);
        self.center();
        self.update_model_pos();
        self.state = BombState::Planted;
    }

    fn slide(&mut self, direction: Direction, delta: f32, world: &BombWorld) {
        let BombWorld {
            map,
            players,
            enemies,
            power_ups,
            bombs_pos,
            burning,
            ..
        } = world;
        let forward = direction.to_vec2();
        let mut budget = delta * 8.0;

        while budget > 1e-4 {
            let tile = tile_of(self.position);
            let to_center = (center_of(tile) - self.position).dot(forward);

            let moved = if to_center > 0.0001 {
                to_center.min(budget)
            } else {
                let ahead = (tile.0 + forward.x as i32, tile.1 + forward.y as i32);
                if !map.is_walkable_tile(ahead.0, ahead.1)
                    || burning.contains(&ahead)
                    || players
                        .iter()
                        .any(|p| p.alive && tile_of(p.position) == ahead)
                    || enemies
                        .iter()
                        .any(|e| e.alive && tile_of(e.position) == ahead)
                    || power_ups.iter().any(|p| p.tile == ahead)
                    || bombs_pos.iter().any(|pos| tile_of(*pos) == ahead)
                {
                    return self.stop_slide(map);
                }
                (1.0 + to_center).min(budget)
            };
            self.position += forward * moved;
            budget -= moved;
        }
        self.update_model_pos();
    }

    pub fn tick(&mut self, delta: f32, world: &mut BombWorld) -> BombEvents {
        if let BombState::Sliding(direction) = self.state {
            self.slide(direction, delta, world);
        }
        let events = match self.state {
            BombState::Planted | BombState::Sliding(_) => self.live_bomb(
                delta,
                world.map,
                world.power_ups,
                world.resources,
                world.audio_manager,
                world.burning,
            ),
            BombState::Exploding => {
                self.exploding_bomb(delta, world.players, world.enemies, world.audio_manager)
            }
        };
        self.enable_collision(world.players);
        if self.despawn {
            world.power_ups.append(&mut self.hidden);
        }
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
    fn get_size(&self) -> f32 {
        match self.state {
            BombState::Exploding => 0.0,
            BombState::Planted => 0.5,
            BombState::Sliding(_) => 0.45,
        }
    }
}
