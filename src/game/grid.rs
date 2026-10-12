use crate::game::{
    bomb::{Bomb, BombState},
    direction::Direction,
    map::{map::Map, map_element::MapElement},
};
use crate::graphics::transform::Transform;
use glam::{Vec2, Vec3};
use std::collections::VecDeque;

pub type Tile = (i32, i32);

pub fn tile_of(pos: Vec2) -> Tile {
    (pos.x.floor() as i32, pos.y.floor() as i32)
}

pub fn center_of(tile: Tile) -> Vec2 {
    Vec2::new(tile.0 as f32 + 0.5, tile.1 as f32 + 0.5)
}

pub fn tile_transform(x: usize, y: usize, height: f32, scale: f32) -> Transform {
    Transform {
        translation: Vec3::new(x as f32 + 0.5, height, y as f32 + 0.5),
        scale: Vec3::splat(scale),
        rotation: Vec3::ZERO,
    }
}

pub fn step(tile: Tile, direction: Direction) -> Tile {
    let v = direction.to_vec2();
    (tile.0 + v.x as i32, tile.1 + v.y as i32)
}

pub fn manhattan(a: Tile, b: Tile) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Free,
    Soft,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridBomb {
    pub tile: Tile,
    pub power: u8,
    pub fuse: f32,
    pub exploding: bool,
}

#[derive(Debug, Clone)]
pub struct TileGrid {
    pub width: i32,
    pub height: i32,
    cells: Vec<Cell>,
    pub bombs: Vec<GridBomb>,
}

#[derive(Debug, Clone)]
pub struct Paths {
    width: i32,
    distance: Vec<u32>,
    parent: Vec<Option<Tile>>,
    pub order: Vec<Tile>,
}

impl Paths {
    pub fn distance(&self, tile: Tile) -> Option<u32> {
        if tile.0 < 0 || tile.1 < 0 || tile.0 >= self.width {
            return None;
        }
        self.distance
            .get((tile.1 * self.width + tile.0) as usize)
            .copied()
            .filter(|d| *d != u32::MAX)
    }

    pub fn first_step(&self, goal: Tile) -> Option<Tile> {
        self.distance(goal)?;
        let mut current = goal;
        while let Some(parent) = self.parent[(current.1 * self.width + current.0) as usize] {
            if self.parent[(parent.1 * self.width + parent.0) as usize].is_none() {
                return Some(current);
            }
            current = parent;
        }
        Some(current)
    }
}

impl TileGrid {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::Free; (width * height).max(0) as usize],
            bombs: Vec::new(),
        }
    }

    pub fn capture(map: &Map, bombs: &[Bomb]) -> Self {
        let mut grid = Self::new(map.width as i32, map.height as i32);
        for y in 0..map.height {
            for x in 0..map.width {
                grid.cells[y * map.width + x] = match map.get_elem(x, y) {
                    MapElement::Empty | MapElement::Exit(_) => Cell::Free,
                    MapElement::Breakable(_) => Cell::Soft,
                    MapElement::Unbreakable(_) => Cell::Hard,
                };
            }
        }
        grid.bombs = bombs
            .iter()
            .map(|bomb| GridBomb {
                tile: tile_of(bomb.position),
                power: bomb.power,
                fuse: if bomb.state == BombState::Exploding {
                    0.0
                } else {
                    bomb.timer
                },
                exploding: bomb.state == BombState::Exploding,
            })
            .collect();
        for tile in bombs.iter().flat_map(|bomb| bomb.burning_tiles()) {
            grid.set_cell(tile, Cell::Soft);
        }
        grid
    }

    pub fn index(&self, tile: Tile) -> Option<usize> {
        (tile.0 >= 0 && tile.1 >= 0 && tile.0 < self.width && tile.1 < self.height)
            .then_some((tile.1 * self.width + tile.0) as usize)
    }

    pub fn cell(&self, tile: Tile) -> Cell {
        self.index(tile).map_or(Cell::Hard, |i| self.cells[i])
    }

    pub fn set_cell(&mut self, tile: Tile, cell: Cell) {
        if let Some(i) = self.index(tile) {
            self.cells[i] = cell;
        }
    }

    pub fn bomb_at(&self, tile: Tile) -> Option<&GridBomb> {
        self.bombs.iter().find(|b| b.tile == tile && !b.exploding)
    }

    pub fn walkable(&self, tile: Tile) -> bool {
        self.cell(tile) == Cell::Free && self.bomb_at(tile).is_none()
    }

    pub fn with_bomb(&self, tile: Tile, power: u8, fuse: f32) -> Self {
        let mut grid = self.clone();
        grid.bombs.push(GridBomb {
            tile,
            power,
            fuse,
            exploding: false,
        });
        grid
    }

    pub fn for_each_blast_tile(&self, origin: Tile, power: u8, mut f: impl FnMut(Tile)) {
        if self.index(origin).is_none() {
            return;
        }
        f(origin);
        for direction in Direction::iterator() {
            let mut tile = origin;
            for _ in 0..power {
                tile = step(tile, *direction);
                match self.cell(tile) {
                    Cell::Hard => break,
                    Cell::Soft => {
                        f(tile);
                        break;
                    }
                    Cell::Free => f(tile),
                }
            }
        }
    }

    fn explosion_times(&self) -> Vec<f32> {
        let mut times: Vec<f32> = self.bombs.iter().map(|b| b.fuse).collect();
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..self.bombs.len() {
                let time = times[i];
                let mut caught = Vec::new();
                self.for_each_blast_tile(self.bombs[i].tile, self.bombs[i].power, |tile| {
                    caught.push(tile)
                });
                for (j, other) in self.bombs.iter().enumerate() {
                    if times[j] > time && caught.contains(&other.tile) {
                        times[j] = time;
                        changed = true;
                    }
                }
            }
        }
        times
    }

    pub fn blast_times(&self) -> Vec<f32> {
        let mut danger = vec![f32::INFINITY; self.cells.len()];
        for (bomb, time) in self.bombs.iter().zip(self.explosion_times()) {
            self.for_each_blast_tile(bomb.tile, bomb.power, |tile| {
                if let Some(i) = self.index(tile) {
                    danger[i] = danger[i].min(time);
                }
            });
        }
        danger
    }

    pub fn bfs(&self, start: Tile, max_depth: u32, passable: impl Fn(Tile, u32) -> bool) -> Paths {
        let len = self.cells.len();
        let mut paths = Paths {
            width: self.width,
            distance: vec![u32::MAX; len],
            parent: vec![None; len],
            order: Vec::new(),
        };
        let Some(start_index) = self.index(start) else {
            return paths;
        };
        paths.distance[start_index] = 0;
        paths.order.push(start);
        let mut queue = VecDeque::from([start]);
        while let Some(tile) = queue.pop_front() {
            let depth = paths.distance[self.index(tile).unwrap_or(0)];
            if depth >= max_depth {
                continue;
            }
            for direction in Direction::iterator() {
                let next = step(tile, *direction);
                let Some(i) = self.index(next) else {
                    continue;
                };
                if paths.distance[i] != u32::MAX
                    || !self.walkable(next)
                    || !passable(next, depth + 1)
                {
                    continue;
                }
                paths.distance[i] = depth + 1;
                paths.parent[i] = Some(tile);
                paths.order.push(next);
                queue.push_back(next);
            }
        }
        paths
    }
}
