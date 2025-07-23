use super::map::Map;
use super::player::Player;
use std::vec::Vec;

#[derive(Default)]
pub struct State {
    players: Vec<Player>,
    map: Map,
}

impl State {
    pub fn new() -> Self {
        State {
            players: Vec::<Player>::new(),
            map: Map::new(16, 16),
        }
    }
    pub fn print(&self) {
        print!("{}", self.map.to_str());
    }
}
