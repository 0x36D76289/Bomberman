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
    pub entities: Vec<EntityType>,
}
impl Zone {
    /// Explore the different neighbouring cells of start and add them if they're not already there.
    /// also add to entities the players encountered
    pub fn fill_zone(mut self, start: Vec2, players: &[Player], map: &Map) -> Self {
        let players_position: Vec<Vec2> = players.iter().map(Player::get_pos).collect();
        self.filling_zone(start, players, &players_position, map);
        self
    }

    pub fn check_bombs(mut self) {}

    pub fn check_powerup(mut self) {}
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
            .filter(|neighbour| self.cells.contains(&neighbour))
            .collect();
        for neighbour in neighbours {
            self.cells.push(neighbour);
            if let Some(player_id) = players_position.iter().position(|pos| *pos == neighbour) {
                self.entities.push(EntityType::Player(player_id as i32));
            }
            self.filling_zone(neighbour, players, players_position, map);
        }
    }
}
