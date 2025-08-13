use std::slice::Iter;

use glam::{Vec2, Vec3, bool};

use crate::{
    game::resources::{ResourceName, Resources},
    graphics::{
        object::{Object, TextureIndex},
        transform::Transform,
    },
};

#[derive(Clone, Debug)]
pub enum MapElement {
    Empty,
    Breakable(Object),
    Unbreakable(Object),
}

impl MapElement {
    fn value(&self) -> char {
        match *self {
            MapElement::Empty => ' ',
            MapElement::Breakable(_) => '#',
            MapElement::Unbreakable(_) => 'X',
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct MapElement {
//     state: MapElementState,
//     pub object: Option<Object>,
// }

pub struct Map {
    pub width: usize,
    pub height: usize,
    content: Vec<MapElement>,
    pub floor: Object,
}

impl Map {
    pub fn new(width: u8, height: u8, ressources: &Resources) -> Self {
        let breakable_object = Object {
            model: ressources.models[ResourceName::Breakable as usize].clone(),
            texture: Some(ResourceName::Breakable as TextureIndex),
            transform: Default::default(),
            color: Default::default(),
        };

        let unbreakable_object = Object {
            model: ressources.models[ResourceName::Unbreakable as usize].clone(),
            texture: Some(ResourceName::Unbreakable as TextureIndex),
            transform: Default::default(),
            color: Default::default(),
        };

        let mut content: Vec<MapElement> = vec![
            MapElement::Breakable(breakable_object.clone());
            width as usize * height as usize
        ];
        for x in 0..width {
            content[x as usize] = MapElement::Unbreakable(unbreakable_object.clone());
            content[width as usize * (height - 1) as usize + x as usize] = MapElement::Unbreakable(unbreakable_object.clone());
        }
        for y in 0..height {
            content[y as usize * width as usize] = MapElement::Unbreakable(unbreakable_object.clone());
            content[(y + 1) as usize * width as usize - 1] = MapElement::Unbreakable(unbreakable_object.clone());
        }
        for y in (0..height).step_by(2) {
            for x in (0..width).step_by(2) {
                content[y as usize * width as usize + x as usize] = MapElement::Unbreakable(unbreakable_object.clone());
            }
        }
        // Top left corner
        content[(1 * width) as usize + 1 as usize] = MapElement::Empty;
        content[(1 * width) as usize + 2 as usize] = MapElement::Empty;
        content[(2 * width) as usize + 1 as usize] = MapElement::Empty;
        // Top right corner
        content[(1 * width) as usize + (width - 2) as usize] = MapElement::Empty;
        content[(1 * width) as usize + (width - 3) as usize] = MapElement::Empty;
        content[(2 * width) as usize + (width - 2) as usize] = MapElement::Empty;
        // Bottom left corner
        content[((height - 2) * width) as usize + 1 as usize] = MapElement::Empty;
        content[((height - 2) * width) as usize + 2 as usize] = MapElement::Empty;
        content[((height - 3) * width) as usize + 1 as usize] = MapElement::Empty;
        // Bottom right corner
        content[((height - 2) * width) as usize + (width - 2) as usize] = MapElement::Empty;
        content[((height - 2) * width) as usize + (width - 3) as usize] = MapElement::Empty;
        content[((height - 3) * width) as usize + (width - 2) as usize] = MapElement::Empty;

        for y in 0..height {
            for x in 0..width {
                match &mut content[y as usize * width as usize + x as usize]
                {
                    MapElement::Empty => (),
                    MapElement::Breakable(obj) => {
                        obj.transform = Transform {
                            translation: Vec3::new(x as f32, 0.0, y as f32),
                            scale: Vec3::splat(0.9),
                            rotation: Vec3::ZERO,
                        }
                    }
                    MapElement::Unbreakable(obj) => {
                        obj.transform = Transform {
                            translation: Vec3::new(x as f32, 0.0, y as f32),
                            scale: Vec3::ONE,
                            rotation: Vec3::ZERO,
                        }
                    }
                }
            }
        }

        let floor = Object {
            model: ressources.models[ResourceName::Floor as usize].clone(),
            texture: Some(ResourceName::Floor as i32),
            transform: Transform {
                translation: Vec3::new(width as f32 / 2.0, 0.0, height as f32 / 2.0),
                scale: Vec3::new(width as f32, 1.0, height as f32),
                rotation: Vec3::ZERO
            },
            color: Vec3::ONE
        };

        Map {
            width: width as usize,
            height: height as usize,
            content: content,
            floor
        }
    }

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
        return Ok(());
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
