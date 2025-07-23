#[derive(Clone, Debug)]
enum MapElement {
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
    width: usize,
    height: usize,
    content: Vec<MapElement>,
}

impl Map {
    pub fn new(width: u8, height: u8) -> Self {
        let mut content: Vec<MapElement> =
            vec![MapElement::Empty; width as usize * height as usize];
        for x in 0..width {
            content[x as usize] = MapElement::Unbreakable;
            content[width as usize * (height - 1) as usize + x as usize] = MapElement::Unbreakable;
        }
        for y in 0..height {
            content[y as usize * width as usize] = MapElement::Unbreakable;
            content[(y + 1) as usize * width as usize - 1] = MapElement::Unbreakable;
        }
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
}

impl Default for Map {
    fn default() -> Self {
        Map::new(16, 16)
    }
}
