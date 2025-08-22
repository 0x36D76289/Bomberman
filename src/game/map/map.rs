use glam::{Vec2, Vec3, usize};
use rand::random_range;

use crate::{
    game::{
        direction::Direction,
        map::{
            map_element::MapElement,
            map_settings::{MapSettings, MapType},
        },
        resources::{ResourceName, Resources},
    },
    graphics::{
        object::{Object, TextureIndex},
        transform::Transform,
    },
};

pub struct Map {
    pub width: usize,
    pub height: usize,
    content: Vec<MapElement>,
    pub floor: Object,
}

impl Map {
    fn create_breakable(ressources: &Resources) -> Object {
        Object {
            model: ressources.models[ResourceName::Breakable as usize].clone(),
            texture: Some(ResourceName::Breakable as TextureIndex),
            transform: Default::default(),
            color: Default::default(),
        }
    }

    fn create_unbreakable(ressources: &Resources) -> Object {
        Object {
            model: ressources.models[ResourceName::Unbreakable as usize].clone(),
            texture: Some(ResourceName::Unbreakable as TextureIndex),
            transform: Default::default(),
            color: Default::default(),
        }
    }

    fn fix_objects(mut self) -> Self {
        for y in 0..self.height {
            for x in 0..self.width {
                match &mut self.content[y * self.width + x] {
                    MapElement::Empty => (),
                    MapElement::SpawnPoint(_) => (),
                    MapElement::Breakable(obj) => {
                        obj.transform = Transform {
                            translation: Vec3::new(x as f32 + 0.5, 0.0, y as f32 + 0.5),
                            scale: Vec3::splat(0.9),
                            rotation: Vec3::ZERO,
                        }
                    }
                    MapElement::Unbreakable(obj) => {
                        obj.transform = Transform {
                            translation: Vec3::new(x as f32 + 0.5, 0.0, y as f32 + 0.5),
                            scale: Vec3::ONE,
                            rotation: Vec3::ZERO,
                        }
                    }
                }
            }
        }
        self
    }

    fn create_floor(width: u8, height: u8, ressources: &Resources) -> Object {
        Object {
            model: ressources.models[ResourceName::Floor as usize].clone(),
            texture: Some(ResourceName::Floor as TextureIndex),
            transform: Transform {
                translation: Vec3::new(width as f32 / 2.0, 0.0, height as f32 / 2.0),
                scale: Vec3::new(width as f32, 1.0, height as f32),
                rotation: Vec3::ZERO,
            },
            color: Vec3::ONE,
        }
    }

    fn empty(width: u8, height: u8, ressources: &Resources) -> Self {
        Map {
            width: width as usize,
            height: height as usize,
            content: vec![MapElement::Empty; width as usize * height as usize],
            floor: Self::create_floor(width, height, ressources),
        }
    }

    fn grid(mut self, ressources: &Resources) -> Self {
        let unbreakable_object = Self::create_unbreakable(ressources);

        for y in (1..self.height).step_by(2) {
            for x in (1..self.width).step_by(2) {
                self.content[y * self.width + x] =
                    MapElement::Unbreakable(unbreakable_object.clone());
            }
        }
        self
    }

    fn fill_break(mut self, ressources: &Resources) -> Self {
        let breakable_object = Self::create_breakable(ressources);

        let mut content_clone = self.content.clone();
        for (i, elem) in content_clone.iter_mut().enumerate() {
            if let MapElement::Empty = elem {
                self.content[i] = MapElement::Breakable(breakable_object.clone())
            }
        }
        self
    }

    fn walls(self, ressources: &Resources) -> Self {
        let unbreakable_object = Self::create_unbreakable(ressources);

        let mut content =
            vec![MapElement::Unbreakable(unbreakable_object); (self.width + 2) * (self.height + 2)];

        for y in 0..self.height {
            for x in 0..self.width {
                content[(y + 1) * (self.width + 2) + x + 1] =
                    self.content[y * self.width + x].clone();
            }
        }
        Self {
            width: self.width + 2,
            height: self.height + 2,
            content,
            floor: Self::create_floor(self.width as u8 + 2, self.height as u8 + 2, ressources),
        }
    }

    fn corners(mut self) -> Self {
        // Top left corner
        self.content[0] = MapElement::SpawnPoint(Direction::Right);
        self.content[1] = MapElement::Empty;
        self.content[self.width] = MapElement::Empty;
        // Top right corner
        self.content[self.width - 1] = MapElement::SpawnPoint(Direction::Left);
        self.content[self.width - 2] = MapElement::Empty;
        self.content[2 * self.width - 1] = MapElement::Empty;
        // Bottom left corner
        self.content[(self.height - 1) * self.width] = MapElement::SpawnPoint(Direction::Right);
        self.content[((self.height - 1) * self.width) + 1] = MapElement::Empty;
        self.content[(self.height - 2) * self.width] = MapElement::Empty;
        // Bottom right corner
        self.content[((self.height - 1) * self.width) + (self.width - 1)] =
            MapElement::SpawnPoint(Direction::Left);
        self.content[((self.height - 1) * self.width) + (self.width - 2)] = MapElement::Empty;
        self.content[((self.height - 2) * self.width) + (self.width - 1)] = MapElement::Empty;

        self
    }

    fn cheese(mut self, cheesiness: u8) -> Self {
        for i in 0..self.content.len() {
            if let MapElement::Breakable(_) = self.content[i]
                && random_range(0..=100) > (100 - cheesiness.clamp(0, 100))
            {
                self.content[i] = MapElement::Empty;
            }
        }
        self
    }

    fn add_spawn(&mut self, safe_range: u8, spawn_size: u8) -> bool {
        if (self.width < 4) || (self.height < 4) {
            return false;
        }
        let mut x: i16 = 0;
        let mut y: i16 = 0;
        while (x % 2) == (y % 2) {
            x = random_range(0..self.width) as i16;
            y = random_range(0..self.height) as i16;
        }
        for y in
            (y - safe_range as i16).max(0)..=(y + safe_range as i16).min(self.height as i16 - 1)
        {
            for x in
                (x - safe_range as i16).max(0)..=(x + safe_range as i16).min(self.width as i16 - 1)
            {
                if let MapElement::SpawnPoint(_) =
                    self.content[y as usize * self.width + x as usize]
                {
                    return false;
                }
            }
        }
        self.content[y as usize * self.width + x as usize] = MapElement::random_spawn_point();
        for y in
            (y - spawn_size as i16).max(0)..=(y + spawn_size as i16).min(self.height as i16 - 1)
        {
            for x in
                (x - spawn_size as i16).max(0)..=(x + spawn_size as i16).min(self.width as i16 - 1)
            {
                if let MapElement::Breakable(_) = self.content[y as usize * self.width + x as usize]
                {
                    self.content[y as usize * self.width + x as usize] = MapElement::Empty
                }
            }
        }
        true
    }

    fn random(mut self, settings: &MapSettings) -> Option<Self> {
        let mut attempts = settings.attempts;
        let mut spawns = settings.spawns;
        while (attempts != 0) && (spawns != 0) {
            if self.add_spawn(settings.safe_range, settings.spawn_size) {
                spawns -= 1;
            } else {
                attempts -= 1;
            }
        }
        if attempts == 0 {
            return None;
        }
        Some(self)
    }

    /// Return neighbouring empty cells
    pub fn get_neighbours(self, position: Vec2) -> Vec<Vec2> {
        Direction::iterator()
            .map(|dir| position + dir.to_vec2())
            .filter(|neighbour_pos| self.get_elem_pos(*neighbour_pos) == &MapElement::Empty)
            .collect()
    }

    pub fn new(settings: MapSettings, ressources: &Resources) -> Option<Self> {
        let mut ret = Self::empty(settings.width, settings.height, ressources)
            .grid(ressources)
            .fill_break(ressources)
            .cheese(settings.cheesiness);

        ret = match settings.map_type {
            MapType::Corners => Some(ret.corners()),
            MapType::Random => ret.random(&settings),
        }?;

        if settings.walls {
            ret = ret.walls(ressources);
        }
        Some(ret.fix_objects())
    }

    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn to_str(&self) -> String {
        let mut str: String = Default::default();
        for y in 0..self.height {
            for x in 0..self.width {
                str.push(self.content[y * self.width + x].value());
            }
            str.push('\n');
        }
        str
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

    pub fn set_elem(&mut self, x: usize, y: usize, elem: MapElement) -> Result<(), ()> {
        if x >= self.width || y >= self.height {
            return Err(());
        }
        self.content[y * self.width + x] = elem;
        // TODO: update object
        Ok(())
    }

    pub fn set_elem_pos(&mut self, pos: Vec2, elem: MapElement) -> Result<(), ()> {
        if pos.x < 0.0 || pos.y < 0.0 {
            return Err(());
        }
        self.set_elem(pos.x as usize, pos.y as usize, elem)
    }

    pub fn iter(&self) -> impl Iterator<Item = &MapElement> {
        self.content.iter()
    }
}
