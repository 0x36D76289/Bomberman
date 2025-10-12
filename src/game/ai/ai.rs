use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use glam::Vec2;

use crate::{
    game::{
        ai::{cpu::CPU, zone::Zone},
        map::map::Map,
        player::Player,
    },
    utils::vec2::Grid,
};

#[derive(Debug, Clone, Eq, PartialEq)]
struct PathNode {
    position: (i32, i32),
    f_cost: i32, // Total cost
    g_cost: i32, // Cost from start
    parent: (i32, i32),
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct AI {}

impl AI {
    pub fn update_zone(cpus: &mut [CPU], players: &[Player], map: &Map) {
        let mut treated_cpu: Vec<usize> = Vec::new();

        for i in 0..cpus.len() {
            if treated_cpu.contains(&cpus[i].id) {
                continue;
            }

            let players_in_zone = cpus[i].update_zone(cpus[i].id, players, map);
            let updated_zone: Arc<Mutex<Zone>> = Arc::clone(&cpus[i].zone);
            treated_cpu.push(cpus[i].id);

            for j in (i + 1)..cpus.len() {
                if players_in_zone.contains(&cpus[j].id) {
                    cpus[j].zone = Arc::clone(&updated_zone);
                    treated_cpu.push(cpus[j].id);
                }
            }
        }
    }

    fn calculate_heuristic(from: (i32, i32), to: (i32, i32)) -> i32 {
        (from.0 - to.0).abs() + (from.1 - to.1).abs()
    }

    pub fn find_path(start: Vec2, goal: Vec2, map: &Map) -> Option<Vec<Vec2>> {
        let start_grid = (start.x as i32, start.y as i32);
        let goal_grid = (goal.x as i32, goal.y as i32);

        if start_grid == goal_grid {
            return Some(vec![]);
        }

        let mut open_list = BinaryHeap::new();
        let mut came_from: HashMap<(i32, i32), ((i32, i32), i32)> = HashMap::new();

        let start_g_cost = 0;
        let start_h_cost = Self::calculate_heuristic(start_grid, goal_grid);

        open_list.push(PathNode {
            position: start_grid,
            f_cost: start_g_cost + start_h_cost,
            g_cost: start_g_cost,
            parent: start_grid,
        });

        came_from.insert(start_grid, (start_grid, start_g_cost));

        while let Some(current_node) = open_list.pop() {
            if current_node.position == goal_grid {
                let mut path = Vec::new();
                let mut current = goal_grid;
                while current != start_grid {
                    path.push(Vec2::new(current.0 as f32 + 0.5, current.1 as f32 + 0.5));
                    current = came_from[&current].0;
                }
                path.reverse();
                return Some(path);
            }

            let neighbours_pos = map.get_neighbours(Vec2::new(
                current_node.position.0 as f32,
                current_node.position.1 as f32,
            ));

            for neighbour_vec in neighbours_pos {
                let neighbour_pos = (neighbour_vec.x as i32, neighbour_vec.y as i32);
                let tentative_g_cost = current_node.g_cost + 1;

                if !came_from.contains_key(&neighbour_pos)
                    || tentative_g_cost < came_from[&neighbour_pos].1
                {
                    let h_cost = Self::calculate_heuristic(neighbour_pos, goal_grid);
                    let f_cost = tentative_g_cost + h_cost;

                    came_from.insert(neighbour_pos, (current_node.position, tentative_g_cost));
                    open_list.push(PathNode {
                        position: neighbour_pos,
                        f_cost,
                        g_cost: tentative_g_cost,
                        parent: current_node.position,
                    });
                }
            }
        }
        None
    }
}
