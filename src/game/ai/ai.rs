use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::game::{
    ai::{cpu::CPU, zone::Zone},
    map::map::Map,
    player::Player,
};

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
}
