use crate::game::ai::entity::Entity;
use crate::game::collision::Collision;
use crate::game::map::map::Map;
use crate::game::player::Player;
use crate::game::{ai::entity::EntityType, game_state::GameState};
use crate::utils::vec2::ApproxEq;
use glam::Vec2;
#[derive(Debug, Clone, Default)]
/// Zone is a collection of empty cells, used mostly for the AI
/// it contains also a vector of entities
pub struct Zone {
    pub cells: Vec<Vec2>,
    pub entities: Vec<Entity>,
}
impl Zone {
    /// Explore the different neighbouring cells of start and add them if they're not already there.
    /// return a vector of the id of every players encountered.
    pub fn fill_zone(&mut self, start: Vec2, players: &[Player], map: &Map) -> Vec<usize> {
        log::debug!("{start:?}");
        if !self.cells.contains(&start) {
            let players_position: Vec<Vec2> = players.iter().map(Player::get_pos).collect();
            self.add_cell(start, players, &players_position);
            self.filling_zone(start, players, &players_position, map);
            log::debug!("{self:?}");
            Entity::get_players_from_list(&self.entities)
        } else {
            Vec::new()
        }
    }

    pub fn check_bombs(&mut self) {
        todo!("Bonjour")
    }

    pub fn check_powerup(&mut self) {
        todo!("Bonjour")
    }
    //
    fn filling_zone(
        &mut self,
        start: Vec2,
        players: &[Player],
        players_position: &[Vec2],
        map: &Map,
    ) {
        let neighbours: Vec<Vec2> = map
            .get_neighbours(start)
            .into_iter()
            .filter(|neighbour| !self.cells.contains(&neighbour))
            .collect();
        for neighbour in neighbours {
            self.add_cell(neighbour, players, players_position);
            self.filling_zone(neighbour, players, players_position, map);
        }
    }

    fn add_cell(&mut self, cell: Vec2, players: &[Player], players_position: &[Vec2]) {
        self.cells.push(cell);
        if let Some(player_id) = players_position.iter().position(|pos| *pos == cell) {
            let new_player = Entity::new(EntityType::Player(player_id as usize));
            if self.entities.iter().all(|entity| *entity != new_player) {
                self.entities.push(new_player);
            }
        }
    }
}
