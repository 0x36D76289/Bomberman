use glam::{Vec3, Vec4};
use vulkano::command_buffer::allocator::{CommandBufferAllocator, StandardCommandBufferAllocator};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract};
use vulkano::device::Queue;
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::StandardMemoryAllocator;

use crate::graphics::{Camera, GameObject, Light, Model, Transform};
use crate::input::{InputState, KeyboardMovementController};

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
    pub textures: Vec<Arc<ImageView>>,
    pub camera: Camera,
    pub camera_controller: KeyboardMovementController,
    pub controlled_object_id: usize,
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
        objects: Vec<GameObject>,
        textures: Vec<Arc<ImageView>>
    ) -> Result<Self, Box<dyn Error>> {
        let input_state = InputState::default();

        let players = Vec::new();

        let map = Map::new(16, 16);
    
        let mut camera = Camera::new();
        camera.set_view_target(Vec3::new(1.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 0.0));

        let mut viewer_object = GameObject::new();
        viewer_object.transform.translation.z = -2.5;

        let camera_controller = KeyboardMovementController {
            move_speed: 2.0,
            look_speed: 2.0,
        };

        let mut light = Light {
            transform: Transform::default(),
            color: Vec4::splat(1.0),
        };
        light.transform.translation = Vec3::splat(-1.0);

        Ok(Self {
            input_state,
            players,
            map,
            objects,
            textures,
            camera,
            camera_controller,
            controlled_object_id: 0,
            light,
        })
    }

    pub fn print(&self) {
        print!("{}", self.map.to_str());
    }
}
