use glam::{bool, Vec2};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum MapElement {
    Empty,
    Breakable,
    Unbreakable,
}

impl MapElement {
    fn value(&self) -> char {
        match *self {
            MapElement::Empty => ' ',
            MapElement::Breakable => '#',
            MapElement::Unbreakable => 'X',
        }
    }
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub content: Vec<MapElement>,
}

impl Map {
    pub fn new(width: u8, height: u8) -> Self {
        let mut content: Vec<MapElement> =
            vec![MapElement::Breakable; width as usize * height as usize];
        for x in 0..width {
            content[x as usize] = MapElement::Unbreakable;
            content[width as usize * (height - 1) as usize + x as usize] = MapElement::Unbreakable;
        }
        for y in 0..height {
            content[y as usize * width as usize] = MapElement::Unbreakable;
            content[(y + 1) as usize * width as usize - 1] = MapElement::Unbreakable;
        }
        for y in (0..height).step_by(2) {
            for x in (0..width).step_by(2) {
                content[y as usize * width as usize + x as usize] = MapElement::Unbreakable;
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
        Map {
            width: width as usize,
            height: height as usize,
            content: content,
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

    pub fn get_elem(&self, x: usize, y: usize) -> MapElement {
        if x >= self.width || y >= self.height {
            return MapElement::Unbreakable;
        }
        self.content[y * self.width + x]
    }

    pub fn get_elem_pos(&self, pos: Vec2) -> MapElement {
        if pos.x < 0.0 || pos.y < 0.0 {
            return MapElement::Unbreakable;
        }
        self.get_elem(pos.x as usize, pos.y as usize)
    }

    pub fn set_elem(&mut self, x: usize, y: usize, elem: MapElement) -> Result<(), ()> {
        if x >= self.width || y >= self.height {
            return Err(());
        }
        self.content[y * self.width + x] = elem;
        return Ok(());
    }

    pub fn set_elem_pos(&mut self, pos: Vec2, elem: MapElement) -> Result<(), ()> {
        if pos.x < 0.0 || pos.y < 0.0 {
            return Err(());
        }
        self.set_elem(pos.x as usize, pos.y as usize, elem)
    }
}

impl Default for Map {
    fn default() -> Self {
        Map::new(15, 15)
    }
}
