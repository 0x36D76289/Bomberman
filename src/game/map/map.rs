use glam::{Vec2, Vec3};
use rand::{RngExt, SeedableRng, random_range, rngs::StdRng, seq::SliceRandom};

use crate::{
    game::{
        direction::Direction,
        enemy::EnemyKind,
        grid::tile_transform,
        map::{
            map_element::{MapElement, SpawnPoint},
            map_settings::{MapSettings, MapType},
        },
        powerup::PowerUpType,
        resources::{FLOOR_SIZE, ResourceName, Resources},
        theme::Theme,
    },
    graphics::{object::Object, transform::Transform},
};

const LEVELS: [&str; 5] = [
    include_str!("../../levels/1.level"),
    include_str!("../../levels/2.level"),
    include_str!("../../levels/3.level"),
    include_str!("../../levels/4.level"),
    include_str!("../../levels/5.level"),
];

pub const LEVEL_COUNT: u32 = LEVELS.len() as u32;

const DECOR_MARGIN_X: i32 = 14;
const DECOR_MARGIN_Y: i32 = 10;

#[derive(Debug, Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    content: Vec<MapElement>,
    pub spawns: Vec<SpawnPoint>,
    pub floor: Object,
    pub theme: Theme,
    pub decor: Vec<Object>,
    pub exit: Option<(usize, usize)>,
    pub hidden_bonuses: Vec<((usize, usize), PowerUpType)>,
    pub random_bonuses: bool,
}

struct LevelHeader {
    name: String,
    theme: Theme,
    time_limit: f32,
    bonuses: Vec<PowerUpType>,
}

pub struct LevelData {
    pub map: Map,
    pub player_spawn: Vec2,
    pub enemy_spawns: Vec<(Vec2, Option<EnemyKind>)>,
    pub exit_pos: Vec2,
    pub name: String,
    pub time_limit: f32,
}

impl Map {
    fn create_breakable(resources: &Resources, theme: Theme) -> Object {
        resources.object(
            ResourceName::Crate,
            Transform::default(),
            theme.crate_tint(),
        )
    }

    fn create_unbreakable(resources: &Resources, theme: Theme) -> Object {
        resources.object(
            theme.block_model(),
            Transform::default(),
            theme.block_tint(),
        )
    }

    pub fn create_exit(resources: &Resources, pos: Vec2) -> Object {
        let (x, y) = (pos.x as usize, pos.y as usize);
        resources.object(
            ResourceName::Exit,
            tile_transform(x, y, 0.0, 1.6),
            Vec3::splat(0.35),
        )
    }

    fn fix_object(elem: &mut MapElement, x: usize, y: usize) {
        let (obj, scale) = match elem {
            MapElement::Empty | MapElement::Exit(_) => return,
            MapElement::Breakable(obj) => (obj, 0.9),
            MapElement::Unbreakable(obj) => (obj, 1.0),
        };
        obj.transform = tile_transform(x, y, 0.0, scale);
    }

    fn fix_objects(mut self) -> Self {
        for y in 0..self.height {
            for x in 0..self.width {
                Self::fix_object(&mut self.content[y * self.width + x], x, y);
            }
        }
        self
    }

    fn create_floor(width: usize, height: usize, resources: &Resources, theme: Theme) -> Object {
        resources.object(
            ResourceName::Floor(theme),
            Transform {
                translation: Vec3::new((width / 2) as f32, 0.0, (height / 2) as f32),
                scale: Vec3::ONE,
                rotation: Vec3::ZERO,
            },
            theme.floor_tint(),
        )
    }

    fn create_decor(
        width: usize,
        height: usize,
        resources: &Resources,
        theme: Theme,
    ) -> Vec<Object> {
        let kinds = theme.decor();
        let mut rng = StdRng::seed_from_u64((width * 1000 + height) as u64 + theme as u64);
        let mut decor = Vec::new();
        let max = FLOOR_SIZE as i32 / 2 - 1;
        let (w, h) = (width as i32, height as i32);

        for y in (-DECOR_MARGIN_Y).max(-max)..(h + DECOR_MARGIN_Y).min(max) {
            for x in (-DECOR_MARGIN_X).max(-max)..(w + DECOR_MARGIN_X).min(max) {
                if ((-1..=w).contains(&x) && (-1..=h).contains(&y)) || !rng.random_bool(0.3) {
                    continue;
                }
                decor.push(resources.object(
                    ResourceName::Decor(kinds[rng.random_range(0..kinds.len())]),
                    Transform {
                        translation: Vec3::new(
                            x as f32 + rng.random_range(0.3..0.7),
                            0.0,
                            y as f32 + rng.random_range(0.3..0.7),
                        ),
                        scale: Vec3::splat(rng.random_range(0.8..1.2)),
                        rotation: Vec3::new(0.0, rng.random_range(0.0..std::f32::consts::TAU), 0.0),
                    },
                    theme.decor_tint(),
                ));
            }
        }
        decor
    }

    fn empty(width: u8, height: u8, resources: &Resources, theme: Theme) -> Self {
        Map {
            width: width as usize,
            height: height as usize,
            spawns: vec![],
            content: vec![MapElement::Empty; width as usize * height as usize],
            floor: Self::create_floor(width as usize, height as usize, resources, theme),
            theme,
            decor: Vec::new(),
            exit: None,
            hidden_bonuses: Vec::new(),
            random_bonuses: true,
        }
    }

    fn finish(mut self, resources: &Resources) -> Self {
        self.floor = Self::create_floor(self.width, self.height, resources, self.theme);
        self.decor = Self::create_decor(self.width, self.height, resources, self.theme);
        self.fix_objects()
    }

    fn grid(mut self, resources: &Resources) -> Self {
        let unbreakable_object = Self::create_unbreakable(resources, self.theme);

        for y in (1..self.height).step_by(2) {
            for x in (1..self.width).step_by(2) {
                self.content[y * self.width + x] =
                    MapElement::Unbreakable(unbreakable_object.clone());
            }
        }
        self
    }

    fn fill_break(mut self, resources: &Resources) -> Self {
        let breakable = Self::create_breakable(resources, self.theme);
        for elem in &mut self.content {
            if *elem == MapElement::Empty {
                *elem = MapElement::Breakable(breakable.clone());
            }
        }
        self
    }

    fn walls(self, resources: &Resources) -> Self {
        let mut content =
            vec![
                MapElement::Unbreakable(Self::create_unbreakable(resources, self.theme));
                (self.width + 2) * (self.height + 2)
            ];

        for y in 0..self.height {
            for x in 0..self.width {
                content[(y + 1) * (self.width + 2) + x + 1] =
                    self.content[y * self.width + x].clone();
            }
        }

        Self {
            width: self.width + 2,
            height: self.height + 2,
            spawns: self
                .spawns
                .iter()
                .map(|spawn| SpawnPoint {
                    x: spawn.x + 1,
                    y: spawn.y + 1,
                    ..*spawn
                })
                .collect(),
            content,
            ..self
        }
    }

    fn corner_tiles(&self) -> [usize; 12] {
        let (w, h) = (self.width, self.height);
        [
            0,
            1,
            w,
            w - 1,
            w - 2,
            2 * w - 1,
            (h - 1) * w,
            (h - 1) * w + 1,
            (h - 2) * w,
            h * w - 1,
            h * w - 2,
            (h - 1) * w - 1,
        ]
    }

    fn corners(mut self) -> Self {
        self.spawns.push(SpawnPoint {
            x: 0,
            y: 0,
            direction: Direction::Right,
        });
        self.spawns.push(SpawnPoint {
            x: self.width as i32 - 1,
            y: 0,
            direction: Direction::Left,
        });
        self.spawns.push(SpawnPoint {
            x: 0,
            y: self.height as i32 - 1,
            direction: Direction::Right,
        });
        self.spawns.push(SpawnPoint {
            x: self.width as i32 - 1,
            y: self.height as i32 - 1,
            direction: Direction::Left,
        });
        for i in self.corner_tiles() {
            self.content[i] = MapElement::Empty;
        }
        self
    }

    fn arena(mut self) -> Self {
        let w = self.width as i16;
        let h = self.height as i16;
        let qx = (w - 1) / 4;
        let qy = (h - 1) / 4 + 1;
        for (x, y, direction) in [
            (w / 2, 0, Direction::Down),
            (w / 2, h - 1, Direction::Up),
            (qx, qy, Direction::Up),
            (w - qx - 1, qy, Direction::Up),
            (qx, h - qy - 1, Direction::Down),
            (w - qx - 1, h - qy - 1, Direction::Down),
        ] {
            self.add_spawn(x, y, 1, Some(direction));
        }
        self
    }

    fn cheese(mut self, cheesiness: u8) -> Self {
        for i in 0..self.content.len() {
            if let MapElement::Breakable(_) = self.content[i]
                && random_range(1..=100) <= cheesiness.clamp(0, 100)
            {
                self.content[i] = MapElement::Empty;
            }
        }
        self
    }

    fn clear_range(&mut self, x: i16, y: i16, size: i16) {
        for y in (y - size).max(0)..=(y + size).min(self.height as i16 - 1) {
            for x in (x - size).max(0)..=(x + size).min(self.width as i16 - 1) {
                if let MapElement::Breakable(_) = self.content[y as usize * self.width + x as usize]
                {
                    self.content[y as usize * self.width + x as usize] = MapElement::Empty
                }
            }
        }
    }

    fn add_spawn(&mut self, x: i16, y: i16, spawn_size: u8, direction: Option<Direction>) {
        if let Some(direction) = direction {
            self.spawns.push(SpawnPoint {
                x: x as i32,
                y: y as i32,
                direction,
            });
        } else {
            self.spawns.push(SpawnPoint::init(x as i32, y as i32));
        }
        self.clear_range(x, y, spawn_size as i16);
    }

    fn add_spawn_random(&mut self, safe_range: u8, spawn_size: u8) {
        let candidates: Vec<(i16, i16, u32)> = (0..self.height as i16)
            .flat_map(|y| (0..self.width as i16).map(move |x| (x, y)))
            .filter(|(x, y)| x % 2 != y % 2)
            .filter(|(x, y)| {
                !self
                    .spawns
                    .iter()
                    .any(|s| s.x == *x as i32 && s.y == *y as i32)
            })
            .map(|(x, y)| {
                (
                    x,
                    y,
                    self.spawns
                        .iter()
                        .map(|s| {
                            (s.x - x as i32)
                                .unsigned_abs()
                                .max((s.y - y as i32).unsigned_abs())
                        })
                        .min()
                        .unwrap_or(u32::MAX),
                )
            })
            .collect();
        let safe: Vec<&(i16, i16, u32)> = candidates
            .iter()
            .filter(|(_, _, gap)| *gap > safe_range as u32)
            .collect();
        if let Some((x, y, _)) = if safe.is_empty() {
            let best = candidates.iter().map(|(_, _, gap)| *gap).max();
            let farthest: Vec<&(i16, i16, u32)> = candidates
                .iter()
                .filter(|(_, _, gap)| Some(*gap) == best)
                .collect();
            farthest
                .get(random_range(0..farthest.len().max(1)))
                .copied()
        } else {
            Some(safe[random_range(0..safe.len())])
        } {
            self.add_spawn(*x, *y, spawn_size, None);
        }
    }

    fn random(mut self, settings: &MapSettings) -> Option<Self> {
        if self.width < 4 || self.height < 4 {
            return None;
        }
        for _ in 0..settings.spawns {
            self.add_spawn_random(settings.safe_range, settings.spawn_size);
        }
        Some(self)
    }

    fn teams(mut self, resources: &Resources) -> Self {
        let w = self.width as i16;
        let h = self.height as i16;
        for (x, y, direction) in [
            (3, 0, Direction::Right),
            (w - 4, 0, Direction::Left),
            (3, h - 1, Direction::Right),
            (w - 4, h - 1, Direction::Left),
            (0, 3, Direction::Down),
            (w - 1, 3, Direction::Down),
            (0, h - 4, Direction::Up),
            (w - 1, h - 4, Direction::Up),
        ] {
            self.add_spawn(x, y, 0, Some(direction));
        }

        self.clear_range(w / 2, h / 2, 2);
        for (x, y) in [(1, 1), (w - 2, 1), (1, h - 2), (w - 2, h - 2)] {
            self.clear_range(x, y, 1);
        }
        let breakable = Self::create_breakable(resources, self.theme);
        for i in self.corner_tiles() {
            self.content[i] = MapElement::Breakable(breakable.clone());
        }

        self
    }

    pub fn new(settings: MapSettings, resources: &Resources) -> Option<Self> {
        let mut ret = Self::empty(settings.width, settings.height, resources, settings.theme)
            .grid(resources)
            .fill_break(resources)
            .cheese(settings.cheesiness);

        ret = match settings.map_type {
            MapType::Corners => Some(ret.corners()),
            MapType::Arena => Some(ret.corners().arena()),
            MapType::Teams => Some(ret.teams(resources)),
            MapType::Random => ret.random(&settings),
        }?;

        if settings.walls {
            ret = ret.walls(resources);
        }
        Some(ret.finish(resources))
    }

    pub fn backdrop(resources: &Resources, theme: Theme) -> Self {
        let mut map = Self::empty(21, 13, resources, theme)
            .grid(resources)
            .fill_break(resources)
            .cheese(45)
            .walls(resources)
            .finish(resources);
        map.content
            .iter_mut()
            .filter(|el| matches!(el, MapElement::Breakable(_)))
            .enumerate()
            .for_each(|(i, el)| {
                if i % 3 == 0 {
                    *el = MapElement::Empty;
                }
            });
        map.fix_objects()
    }

    fn parse_level(level: u32) -> Option<(LevelHeader, Vec<Vec<u8>>)> {
        let mut header = LevelHeader {
            name: format!("Niveau {level}"),
            theme: Theme::Grass,
            time_limit: 200.0,
            bonuses: Vec::new(),
        };
        let mut lines = Vec::new();
        for line in LEVELS.get((level as usize).checked_sub(1)?)?.lines() {
            if let Some(line) = line.strip_prefix('@') {
                let (key, value) = line.split_once(' ').unwrap_or((line, ""));
                match key {
                    "name" => header.name = value.trim().to_string(),
                    "theme" => header.theme = Theme::from_name(value).unwrap_or_default(),
                    "time" => header.time_limit = value.trim().parse().unwrap_or(header.time_limit),
                    "bonus" => {
                        header.bonuses = value
                            .split_whitespace()
                            .filter_map(PowerUpType::from_name)
                            .collect()
                    }
                    _ => (),
                }
            } else if !line.is_empty() {
                lines.push(line.as_bytes().to_vec());
            }
        }
        Some((header, lines))
    }

    pub fn level_info(level: u32) -> Option<(String, Theme)> {
        let (header, _) = Self::parse_level(level)?;
        Some((header.name, header.theme))
    }

    pub fn from_file(
        level: u32,
        random_seed: Option<u64>,
        resources: &Resources,
    ) -> Option<LevelData> {
        let (
            LevelHeader {
                name,
                theme,
                time_limit,
                bonuses,
            },
            mut lines,
        ) = Self::parse_level(level)?;

        if let Some(seed) = random_seed {
            randomize_layout(&mut lines, seed.wrapping_add(level as u64));
        }

        let height = lines.len();
        let width = lines.first()?.len();

        let mut content = vec![MapElement::Empty; width * height];
        let mut player_spawn = Vec2::new(1.5, 1.5);
        let mut enemy_spawns = Vec::new();
        let mut exit_pos = None;
        let mut bonus_tiles = Vec::new();

        let breakable_obj = Self::create_breakable(resources, theme);
        let unbreakable_obj = Self::create_unbreakable(resources, theme);

        for (y, line) in lines.iter().enumerate() {
            for (x, char) in line.iter().map(|c| *c as char).enumerate().take(width) {
                let pos = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                content[y * width + x] = match char {
                    'X' => MapElement::Unbreakable(unbreakable_obj.clone()),
                    '#' => MapElement::Breakable(breakable_obj.clone()),
                    '*' => {
                        bonus_tiles.push((x, y));
                        MapElement::Breakable(breakable_obj.clone())
                    }
                    'P' => {
                        player_spawn = pos;
                        MapElement::Empty
                    }
                    'O' => {
                        exit_pos = Some(pos);
                        MapElement::Breakable(breakable_obj.clone())
                    }
                    'E' => {
                        enemy_spawns.push((pos, None));
                        MapElement::Empty
                    }
                    c => {
                        if let Some(kind) = EnemyKind::from_level_char(c) {
                            enemy_spawns.push((pos, Some(kind)));
                        }
                        MapElement::Empty
                    }
                };
            }
        }
        let exit_pos = exit_pos?;

        Some(LevelData {
            map: Map {
                width,
                height,
                content,
                spawns: vec![SpawnPoint {
                    x: player_spawn.x as i32,
                    y: player_spawn.y as i32,
                    direction: Direction::Down,
                }],
                floor: Self::create_floor(width, height, resources, theme),
                theme,
                decor: Vec::new(),
                exit: Some((exit_pos.x as usize, exit_pos.y as usize)),
                hidden_bonuses: bonus_tiles.into_iter().zip(bonuses).collect(),
                random_bonuses: false,
            }
            .finish(resources),
            player_spawn,
            enemy_spawns,
            exit_pos,
            name,
            time_limit,
        })
    }

    pub fn get_elem(&self, x: usize, y: usize) -> &MapElement {
        if x >= self.width || y >= self.height {
            return &MapElement::Empty;
        }
        &self.content[y * self.width + x]
    }

    pub fn get_elem_pos(&self, pos: Vec2) -> &MapElement {
        if pos.x < 0.0 || pos.y < 0.0 {
            return &MapElement::Empty;
        }
        self.get_elem(pos.x as usize, pos.y as usize)
    }

    pub fn get_elem_mut_pos(&mut self, pos: Vec2) -> Option<&mut MapElement> {
        if pos.x < 0.0 || pos.y < 0.0 || pos.x as usize >= self.width {
            return None;
        }
        self.content
            .get_mut(pos.y as usize * self.width + pos.x as usize)
    }

    fn tile(&self, x: i32, y: i32) -> Option<&MapElement> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some(&self.content[y as usize * self.width + x as usize])
    }

    pub fn is_breakable_tile(&self, x: i32, y: i32) -> bool {
        matches!(self.tile(x, y), Some(MapElement::Breakable(_)))
    }

    pub fn is_walkable_tile(&self, x: i32, y: i32) -> bool {
        matches!(
            self.tile(x, y),
            Some(MapElement::Empty | MapElement::Exit(_))
        )
    }

    pub fn find_random_empty(
        &self,
        avoid: &[Vec2],
        min_distance: f32,
        attempts: u32,
    ) -> Option<Vec2> {
        if self.width == 0 || self.height == 0 {
            return None;
        }
        for _ in 0..attempts {
            let x = random_range(0..self.width) as i32;
            let y = random_range(0..self.height) as i32;
            if !self.is_walkable_tile(x, y) {
                continue;
            }
            let pos = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            if avoid
                .iter()
                .any(|other| other.distance_squared(pos) <= min_distance * min_distance)
            {
                continue;
            }
            return Some(pos);
        }
        None
    }

    pub fn set_elem_pos(&mut self, pos: Vec2, mut elem: MapElement) {
        let (x, y) = (pos.x as usize, pos.y as usize);
        if pos.x < 0.0 || pos.y < 0.0 || x >= self.width || y >= self.height {
            return;
        }
        Self::fix_object(&mut elem, x, y);
        self.content[y * self.width + x] = elem;
    }

    pub fn iter(&self) -> impl Iterator<Item = &MapElement> {
        self.content.iter()
    }
}

fn randomize_layout(lines: &mut [Vec<u8>], seed: u64) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut enemies = Vec::new();
    let mut player = (1, 1);
    let mut breakables = 0usize;
    let mut hidden = 0usize;

    for (y, line) in lines.iter_mut().enumerate() {
        for (x, c) in line.iter_mut().enumerate() {
            match *c {
                b'X' => continue,
                b'P' => player = (x, y),
                b'#' | b'O' => breakables += 1,
                b'*' => {
                    breakables += 1;
                    hidden += 1;
                }
                e if e == b'E' || EnemyKind::from_level_char(e as char).is_some() => {
                    enemies.push(e)
                }
                _ => (),
            }
            if *c != b'P' {
                *c = b' ';
            }
        }
    }

    let distance = |x: usize, y: usize| x.abs_diff(player.0) + y.abs_diff(player.1);
    let mut free: Vec<(usize, usize)> = lines
        .iter()
        .enumerate()
        .flat_map(|(y, line)| {
            line.iter()
                .enumerate()
                .filter(|(_, c)| **c == b' ')
                .map(move |(x, _)| (x, y))
        })
        .collect();
    free.shuffle(&mut rng);

    for enemy in enemies {
        if let Some(i) = free.iter().position(|&(x, y)| distance(x, y) >= 6) {
            let (x, y) = free.swap_remove(i);
            lines[y][x] = enemy;
        }
    }
    let mut placed = 0;
    let mut exit_placed = false;
    for (x, y) in free {
        if placed >= breakables || distance(x, y) <= 2 {
            continue;
        }
        if !exit_placed && distance(x, y) >= lines[0].len() / 2 && rng.random_bool(0.3) {
            lines[y][x] = b'O';
            exit_placed = true;
        } else {
            lines[y][x] = b'#';
        }
        placed += 1;
    }
    if !exit_placed
        && let Some((x, y, _)) = lines
            .iter()
            .enumerate()
            .flat_map(|(y, line)| line.iter().enumerate().map(move |(x, c)| (x, y, *c)))
            .filter(|(_, _, c)| *c == b'#')
            .max_by_key(|(x, y, _)| distance(*x, *y))
    {
        lines[y][x] = b'O';
    }
    let mut bricks: Vec<(usize, usize)> = lines
        .iter()
        .enumerate()
        .flat_map(|(y, line)| line.iter().enumerate().map(move |(x, c)| (x, y, *c)))
        .filter(|(_, _, c)| *c == b'#')
        .map(|(x, y, _)| (x, y))
        .collect();
    bricks.shuffle(&mut rng);
    for (x, y) in bricks.into_iter().take(hidden) {
        lines[y][x] = b'*';
    }
}
