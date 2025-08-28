mod app;
mod app_state;
mod audio;
mod game;
mod graphics;
mod input;
mod settings;
mod ui;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::settings::settings::Settings;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let settings = Settings::load_settings();

    let mut app = App::init(settings, &event_loop).unwrap();
    event_loop.run_app(&mut app).unwrap();
}
