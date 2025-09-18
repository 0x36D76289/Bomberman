use crate::{
    app_state::{AppState, KeyMap},
    audio::AudioManager,
    game::{
        Camera,
        bomb::{Bomb, BombState},
        game_settings::GameSettings,
        map::{map::Map, map_element::MapElement, map_settings::MapSettings},
        player::Player,
        powerup::PowerUp,
        resources::Resources,
    },
    graphics::{GlobalUbo, LightInfo, object::Object, transform::Transform},
    input::{input::Input, input_state::InputState, input_vec::GetOrDefault},
    ui::UiState,
};
use glam::{Vec2, Vec3, Vec4, bool};
use rand::random_range;
use std::{error::Error, sync::Mutex, vec::Vec};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone)]
pub struct GameState {
    players: Vec<Player>,
    game_inputs: Vec<Input>,
    nb_humans: u32,
    bombs: Vec<Bomb>,
    power_ups: Vec<PowerUp>,
    map: Map,
    camera: Transform,
    light: LightInfo,
}

impl GameState {
    fn create_players(map: &Map, resources: &Resources, nb_humans: &u32) -> Vec<Player> {
        let mut players = Vec::<Player>::new();
        let mut id: u32 = 0;
        for spawn in map.spawns.clone() {
            players.push(Player::new(
                id,
                Vec2 {
                    x: spawn.x as f32 + 0.5,
                    y: spawn.y as f32 + 0.5,
                },
                spawn.direction,
                resources,
                id < *nb_humans,
            ));

            id += 1;
        }

        players
    }

    //TODO: add get_input_player -> returns Released if p doesn't exist
    pub fn default_state(
        resources: &Resources,
        settings: GameSettings,
    ) -> Result<Self, Box<dyn Error>> {
        //HACK: this is not safe, map can fail creation
        //LOIC: true
        let map = Map::new(settings.map_settings, &resources).unwrap();
        let nb_humans = settings.nb_humans;
        let players = Self::create_players(&map, &resources, &nb_humans);
        let game_inputs = vec![Input::default(); players.len()];

        let camera = Transform {
            translation: Vec3::new(map.width as f32 / 2.0, -1.0, map.height as f32 / 2.0),
            scale: Vec3::ONE,
            rotation: Vec3::new(-1.25, 0.0, 0.0),
        };

        let light = LightInfo {
            ambient_light_color: Vec4::ONE.with_w(0.8),
            direction_to_light: Vec3::new(-1.0, -1.0, 1.0).normalize(),
            directional_light_color: Vec4::ONE.with_w(0.6),
        };

        Ok(Self {
            players,
            game_inputs,
            nb_humans,
            bombs: Vec::<Bomb>::new(),
            power_ups: Vec::<PowerUp>::new(),
            map,
            camera,
            light,
        })
    }

    fn recreate(&self, resources: &Resources) -> Self {
        let map = Map::new(MapSettings::default(), resources).unwrap();
        let players = Self::create_players(&map, &resources, &self.nb_humans);
        Self {
            players,
            game_inputs: self.game_inputs.clone(),
            nb_humans: self.nb_humans,
            bombs: Vec::<Bomb>::new(),
            power_ups: Vec::<PowerUp>::new(),
            map,
            camera: self.camera,
            light: self.light,
        }
    }

    pub fn objects_to_render(&self) -> impl Iterator<Item = &Object> {
        let map_objects = self
            .map
            .iter()
            .filter_map(|el| match el {
                MapElement::Empty => None,
                MapElement::Breakable(obj) => Some(obj),
                MapElement::Unbreakable(obj) => Some(obj),
            })
            .chain(std::iter::once(&self.map.floor));

        let players_objects = self.players.iter().flat_map(|p| &p.object);
        let bomb_objects = self.bombs.iter().flat_map(|b| &b.objects);
        let power_up_objects = self.power_ups.iter().map(|p| &p.object);

        map_objects
            .chain(players_objects)
            .chain(bomb_objects)
            .chain(power_up_objects)
    }

    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn print(&self) {
        let mut display = self.map.to_str();
        for player in &self.players {
            println!("player pos: {} {}", player.position.x, player.position.y);
            let y: usize = player.position.y as usize;
            let x: usize = player.position.x as usize;
            println!("player pos: {} {}", x, y);
            let pos: usize = y * (self.map.width + 1) + x;
            display.replace_range(pos..pos + 1, "+");
        }
        for bomb in &self.bombs {
            let y: usize = bomb.position.y as usize;
            let x: usize = bomb.position.x as usize;
            let pos: usize = y * (self.map.width + 1) + x;
            display.replace_range(pos..pos + 1, "O");
        }

        print!("{}", display);
    }

    fn mp_game_tick(
        &mut self,
        delta: f32,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) {
        // tick bombs
        for bomb in &mut self.bombs {
            bomb.tick(
                delta,
                &mut self.players,
                &mut self.map,
                &mut self.power_ups,
                resources,
                audio_manager,
            );
        }
        for i in 0..self.bombs.len() {
            if self.bombs[i].state != BombState::Exploding {
                continue;
            }
            self.bombs[i].clone().chain_react(&mut self.bombs);
        }
        self.bombs.retain(|bomb| !bomb.despawn);
        //tick powerups
        for powerup in &mut self.power_ups {
            powerup.tick(&mut self.players, audio_manager);
        }
        self.power_ups.retain(|powerup| !powerup.despawn);
        // for player in players: summon bomb if Pressed
        for (i, player) in self.players.iter_mut().enumerate() {
            if !player.alive {
                continue;
            }
            if self.game_inputs.get_or_default(i).bomb() == InputState::Pressed
                && let Some(bomb) = player.create_bomb(&resources, &self.bombs)
            {
                audio_manager.play_sound_effect(crate::audio::SoundEffect::PutBomb);
                self.bombs.push(bomb)
            }
        }
        for (i, player) in self.players.iter_mut().enumerate() {
            player.player_move(
                self.game_inputs.get_or_default(i),
                delta,
                &self.map,
                &mut self.bombs,
            );
        }
        // uncomment this and comment the previous line to control the camera
        // self.camera.keyboard_move(&self.game_inputs[0], delta);
    }

    fn restart_inside(&mut self, keys: &KeyMap, resources: &Resources) {
        static WAS_PRESSED: Mutex<bool> = Mutex::new(false);

        let mut was_pressed = WAS_PRESSED.lock().unwrap();
        let is_pressed = keys
            .get(&PhysicalKey::Code(KeyCode::KeyR))
            .unwrap_or(&winit::event::ElementState::Released)
            .is_pressed();

        if is_pressed && !*was_pressed {
            //HACK: this is the only part that would be kept if this wasn't a silly bind
            self.map = Map::new(
                MapSettings {
                    spawns: random_range(2..=8),
                    ..MapSettings::default_cheese()
                },
                resources,
            )
            .unwrap();
            self.players = Self::create_players(&self.map, resources, &self.nb_humans);
            self.bombs = Vec::new();
        }
        *was_pressed = is_pressed;
    }

    pub fn tick(
        &mut self,
        delta_time: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        //Pause
        if keys
            .get(&PhysicalKey::Code(KeyCode::Escape))
            .unwrap_or(&winit::event::ElementState::Released)
            .is_pressed()
        {
            return (Some(AppState::Ui(UiState::pause())), 0);
        }

        #[cfg(debug_assertions)]
        self.restart_inside(keys, resources);

        // let state_func = match self.mode {
        //     Mode::MpGame => Self::mp_game_tick,
        // };
        self.inputs_to_game_inputs(inputs);
        self.mp_game_tick(delta_time, resources, audio_manager);

        #[cfg(debug_assertions)]
        if keys
            .get(&PhysicalKey::Code(KeyCode::KeyT))
            .unwrap_or(&winit::event::ElementState::Released)
            .is_pressed()
        {
            return (Some(AppState::Game(self.recreate(resources))), 1);
        }

        //TODO: return new AppState if needed and number of elements to pop from appstate_stack
        (None, 0)
    }

    // Put the inputs read into game inputs
    fn inputs_to_game_inputs(&mut self, inputs: &Vec<Input>) {
        for (i, input) in inputs.iter().enumerate() {
            self.game_inputs[i] = input.clone();
        }
    }

    pub fn create_ubo(&self, aspect_ratio: f32) -> GlobalUbo {
        let camera = {
            let mut camera = Camera::new();
            let clip = (self.map.width.max(self.map.height) as f32 / 2.0) * 1.15;
            camera.set_orthographic_projection(
                -clip * aspect_ratio,
                clip * aspect_ratio,
                -clip,
                clip,
                -clip,
                clip * 2.0,
            );
            camera.set_view_xyz(self.camera.translation, self.camera.rotation);
            camera
        };

        let light = {
            let mut light = Camera::new();
            let clip = (self.map.width.max(self.map.height) as f32 / 2.0) * 2.0;
            light.set_orthographic_projection(
                -clip * aspect_ratio,
                clip * aspect_ratio,
                -clip,
                clip,
                -clip,
                clip * 2.0,
            );
            let map_center = Vec3::new(
                self.map.width as f32 / 2.0,
                0.0,
                self.map.height as f32 / 2.0,
            );
            let direction_to_light = self.light.direction_to_light;
            let light_pos = Vec3::new(
                map_center.x + map_center.x * direction_to_light.x,
                direction_to_light.y,
                map_center.z + map_center.z * direction_to_light.z,
            );
            light.set_view_direction(light_pos, direction_to_light * -1.0);
            light
        };

        GlobalUbo {
            projection: camera.projection_matrix.to_cols_array_2d(),
            view: camera.view_matrix.to_cols_array_2d(),
            inverse_view: camera.inverse_view_matrix.to_cols_array_2d(),
            light_view: light.view_matrix.to_cols_array_2d(),
            light_projection: light.projection_matrix.to_cols_array_2d(),
            ambient_light_color: self.light.ambient_light_color.into(),
            direction_to_light: self.light.direction_to_light.to_array().into(),
            directional_light_color: self.light.directional_light_color.into(),
        }
    }

    pub fn get_player(&self, id: u32) -> Option<&Player> {
        self.players.get(id as usize)
    }

    pub fn get_map(&self) -> &Map {
        &self.map
    }
}
