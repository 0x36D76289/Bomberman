use std::collections::HashSet;

use crate::game::ai::entity::Entity;
use crate::game::collision::Collision;
use crate::game::map::map::Map;
use crate::game::player::Player;
use crate::game::powerup::PowerUp;
use crate::game::{ai::entity::EntityType, game_state::GameState};
use crate::utils::vec2::ApproxEq;
use glam::Vec2;
#[derive(Debug, Clone, Default)]
/// Zone is a collection of empty cells, used mostly for the AI
/// it contains also a vector of entities
pub struct Zone {
    pub cells: Vec<Vec2>,
    pub entities: Vec<Entity>,
    player_zone: bool,
}
impl Zone {
    /// Explore the different neighbouring cells of start and add them if they're not already there.
    /// return a vector of the id of every players encountered.
    pub fn fill_zone(&mut self, start: Vec2, players: &[Player], map: &Map) -> Vec<usize> {
        // log::debug!("i'm starting here {start:?}");
        self.player_zone = false;
        let mut cells_found: Vec<Vec2> = [].to_vec();
        let mut entities_found: Vec<Entity> = [].to_vec();
        if !self.cells.contains(&start) {
            let players_position: Vec<Vec2> = players.iter().map(Player::get_pos).collect();
            self.add_cell(
                &mut cells_found,
                &mut entities_found,
                start,
                players,
                &players_position,
            );
            self.filling_zone(
                &mut cells_found,
                &mut entities_found,
                start,
                players,
                &players_position,
                map,
            );
            if self.player_zone || self.cells.is_empty() {
                for new_cell in cells_found {
                    self.cells.push(new_cell);
                }
                for entities in entities_found {
                    self.entities.push(entities);
                }
            }
            Entity::get_players_from_list(&self.entities)
        } else {
            Vec::new()
        }
    }

    pub fn check_bombs(&mut self) {
        todo!("Bonjour")
    }

    pub fn check_powerup(&mut self, power_ups: &[PowerUp]) -> Option<Vec2> {
        let powerup_positions: Vec<Vec2> = power_ups
            .iter()
            .map(|power_up| power_up.pos.as_vec2())
            .collect();
        for &cell in &self.cells {
            if powerup_positions.contains(&cell) {
                return Some(cell);
            }
        }
        None
    }
    //
    fn filling_zone(
        &mut self,
        cells_found: &mut Vec<Vec2>,
        entities_found: &mut Vec<Entity>,
        start: Vec2,
        players: &[Player],
        players_position: &[Vec2],
        map: &Map,
    ) {
        let neighbours: Vec<Vec2> = map
            .get_neighbours(start)
            .into_iter()
            .filter(|neighbour| {
                !cells_found.contains(&neighbour)
                    || (self.player_zone && !self.cells.contains(&neighbour))
            })
            .collect();
        for neighbour in neighbours {
            if self.cells.contains(&neighbour) {
                self.player_zone = true;
                continue;
            }
            self.add_cell(
                cells_found,
                entities_found,
                neighbour,
                players,
                players_position,
            );
            self.filling_zone(
                cells_found,
                entities_found,
                neighbour,
                players,
                players_position,
                map,
            );
        }
    }

    fn add_cell(
        &mut self,
        cells_found: &mut Vec<Vec2>,
        entities_found: &mut Vec<Entity>,
        cell: Vec2,
        players: &[Player],
        players_position: &[Vec2],
    ) {
        // log::debug!("added {cell} :)");
        cells_found.push(cell);
        if let Some(player_id) = players_position.iter().position(|pos| *pos == cell) {
            let new_player = Entity::new(EntityType::Player(player_id as usize));
            if self.entities.iter().all(|entity| *entity != new_player) {
                entities_found.push(new_player);
            }
        }
    }
}
