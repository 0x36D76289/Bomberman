use glam::Vec3;
use vulkano::image::view::ImageView;

use crate::game::{Camera, Entity};
use crate::input::{InputState, KeyboardMovementController};

// use super::map::Map;
// use super::player::Player;
use std::error::Error;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Default)]
pub struct State {
    pub input_state: InputState,
    // pub players: Vec<Player>,
    // pub map: Map,
    pub entities: Vec<Entity>,
    pub textures: Vec<Arc<ImageView>>,
    pub camera: Camera,
    pub entity_controller: KeyboardMovementController,
    pub controlled_object_id: usize,
}

impl State {
    // pub fn new() -> Self {
    //     State {
    //         players: Vec::<Player>::new(),
    //         map: Map::new(16, 16),
    //         ..Default::default()
    //     }
    // }

    pub fn default_state(
        entities: Vec<Entity>,
        textures: Vec<Arc<ImageView>>,
    ) -> Result<Self, Box<dyn Error>> {
        let input_state = InputState::default();

        // let players = Vec::new();

        // let map = Map::new(16, 16);

        let mut camera = Camera::new();
        camera.set_view_target(Vec3::new(1.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0));

        let entity_controller = KeyboardMovementController {
            move_speed: 3.0,
            look_speed: 1.5,
        };

        Ok(Self {
            input_state,
            // players,
            // map,
            entities,
            textures,
            camera,
            entity_controller,
            controlled_object_id: 1,
        })
    }

    pub fn debug(&self) {
        for entity in self.entities.iter() {
            println!("{entity:?}");
        }
    }

    // pub fn print(&self) {
    //     print!("{}", self.map.to_str());
    // }
}
