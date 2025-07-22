#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub framerate_limit: u32,
    pub volume_music: f32,
    pub volume_sfx: f32,
    // pub key_bindings: HashMap<String, KeyCode>,
}
