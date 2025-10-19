use std::collections::HashSet;

use crate::game::ai::ai::AI;
use crate::game::ai::entity::Entity;
use crate::game::bomb::Bomb;
use crate::game::collision::Collision;
use crate::game::map::map::Map;
use crate::game::player::Player;
use crate::game::powerup::PowerUp;
use crate::game::{ai::entity::EntityType, game_state::GameState};
use crate::utils::vec2::{ApproxEq, Grid};
use glam::Vec2;
#[derive(Debug, Clone, Default)]
/// Zone is a collection of empty cells, used mostly for the AI
/// it contains also a vector of entities

pub struct Zone {
    cells: Vec<Vec2>,
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
    pub fn get_accessible_cells(&self, start: Vec2, map: &Map, bombs: &[Bomb]) -> Vec<Vec2> {
        if bombs.is_empty() {
            // log::debug!("bomb empty !:accessible cells : {0:?}=", self.cells);
            return self.cells.clone();
        }
        let mut accessible_cells = Vec::new();
        let bombs_pos: Vec<Vec2> = bombs.iter().map(|bomb| bomb.position).collect();
        self.get_accessible_cells_loop(start, &mut accessible_cells, map, &bombs_pos);
        // log::debug!("accessible cells : {accessible_cells:?}=");
        accessible_cells
    }

    fn get_accessible_cells_loop(
        &self,
        cell: Vec2,
        accessible_cells: &mut Vec<Vec2>,
        map: &Map,
        bombs_pos: &Vec<Vec2>,
    ) {
        let neighbours: Vec<Vec2> = map
            .get_neighbours(cell.grid())
            .into_iter()
            .filter(|neighbour| {
                !accessible_cells.contains(&neighbour) && !bombs_pos.contains(&neighbour)
            })
            .collect();
        for neighbour in neighbours {
            accessible_cells.push(neighbour.grid());
            self.get_accessible_cells_loop(neighbour, accessible_cells, map, bombs_pos);
        }
    }

    pub fn check_dangerous_cells(&mut self, bombs: &[Bomb], map: &Map) -> Vec<Vec2> {
        let mut dangerous_cells: Vec<Vec2> = Vec::new();
        for bomb in bombs {
            dangerous_cells.append(&mut bomb.get_explosion_cells(map))
        }
        dangerous_cells
    }

    // TODO: {loic} create generic function for getting closest position
    pub fn closest_player(&mut self, pos: Vec2, players: &[Player]) -> Option<Vec2> {
        let mut players_position: Vec<Vec2> = players
            .iter()
            .map(|player| player.position)
            .filter(|player_pos| *player_pos != pos)
            .collect();
        players_position.sort_by(|p, p2| {
            let d1 = AI::calculate_heuristic_pos(&pos, p);
            let d2 = AI::calculate_heuristic_pos(&pos, p2);
            d1.cmp(&d2)
        });
        for &player in &players_position {
            if self.cells.contains(&player) {
                return Some(player);
            }
        }
        None
    }
    pub fn closest_powerup(&mut self, pos: Vec2, power_ups: &[PowerUp]) -> Option<Vec2> {
        let mut powerup_positions: Vec<Vec2> = power_ups
            .iter()
            .map(|power_up| power_up.pos.as_vec2())
            .collect();
        powerup_positions.sort_by(|p, p2| {
            let d1 = AI::calculate_heuristic_pos(&pos, p);
            let d2 = AI::calculate_heuristic_pos(&pos, p2);
            d1.cmp(&d2)
        });
        for &powerup in &powerup_positions {
            if self.cells.contains(&powerup) {
                return Some(powerup);
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
        // log::debug!(
        //     "== {} ==\ncells found during zone: {cells_found:?}, cells already in zone {2:?}, is player in zone ? {}\n",
        //     start,
        //     self.player_zone,
        //     self.cells
        // );
        let neighbours: Vec<Vec2> = map
            .get_neighbours(start.grid())
            .into_iter()
            .filter(|neighbour| !cells_found.contains(&neighbour))
            .collect();
        // log::debug!("cells to visit : {neighbours:?}");
        for neighbour in neighbours {
            if self.cells.contains(&neighbour) {
                self.player_zone = true;
                continue;
            }
            // log::debug!("visiting neihggbour : {neighbour:?}");
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
        cells_found.push(cell.grid());
        if let Some(player_id) = players_position.iter().position(|pos| *pos == cell) {
            let new_player = Entity::new(EntityType::Player(player_id as usize));
            if self.entities.iter().all(|entity| *entity != new_player) {
                entities_found.push(new_player);
            }
        }
    }
}
