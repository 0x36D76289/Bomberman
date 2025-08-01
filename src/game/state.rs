use glam::{Vec3, Vec4};
use vulkano::memory::allocator::StandardMemoryAllocator;

use crate::graphics::{Camera, GameObject, Light, Model};
use crate::input::{InputState, KeyboardMovementController};
use crate::load_model;

use super::map::Map;
use super::player::Player;
use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;
use std::vec::Vec;

#[derive(Default)]
pub struct State {
    pub input_state: InputState,
    pub players: Vec<Player>,
    pub map: Map,
    pub objects: Vec<GameObject>,
    pub camera: Camera,
    pub viewer_object: GameObject,
    pub camera_controller: KeyboardMovementController,
    pub light: Light,
}

impl State {
    pub fn new() -> Self {
        State {
            players: Vec::<Player>::new(),
            map: Map::new(16, 16),
            ..Default::default()
        }
    }

    pub fn default_state(
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self, Box<dyn Error>> {
        let input_state = InputState::default();

        let players = Vec::new();

        let map = Map::new(16, 16);

        let objects = load_game_objects(memory_allocator)?;

        let mut camera = Camera::new();
        camera.set_view_target(Vec3::new(1.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0));

        let mut viewer_object = GameObject::new();
        viewer_object.transform.translation.z = -2.5;

        let camera_controller = KeyboardMovementController {
            move_speed: 3.0,
            look_speed: 1.5,
        };

        let light = Light {
            position: Vec3::new(0.0, -1.0, 0.0),
            color: Vec4::splat(1.0),
        };

        Ok(Self {
            input_state,
            players,
            map,
            objects,
            camera,
            viewer_object,
            camera_controller,
            light,
        })
    }

    pub fn print(&self) {
        print!("{}", self.map.to_str());
    }
}

fn load_game_objects(
    memory_allocator: Arc<StandardMemoryAllocator>,
) -> Result<Vec<GameObject>, Box<dyn Error>> {
    let model = load_model!("../assets/miku.obj", memory_allocator);
    let mut miku = GameObject::new();
    miku.model = Some(model.clone());
    miku.transform.translation = Vec3::new(-0.5, 0.5, 0.0);
    miku.transform.scale = Vec3::splat(0.1);
    miku.color = Vec3::new(0.0, 0.0, 1.0);

    let model = load_model!("../assets/link.obj", memory_allocator);
    let mut link = GameObject::new();
    link.model = Some(model.clone());
    link.transform.translation = Vec3::new(0.5, 0.5, 0.0);
    link.transform.scale = Vec3::splat(0.06);
    link.color = Vec3::new(1.0, 0.0, 0.0);

    let model = load_model!("../assets/quad.obj", memory_allocator);
    let mut floor = GameObject::new();
    floor.model = Some(model.clone());
    floor.transform.translation = Vec3::new(0.0, 0.5, 0.0);
    floor.transform.scale = Vec3::new(3.0, 1.0, 3.0);

    let objects = vec![floor, miku, link];

    Ok(objects)
}
