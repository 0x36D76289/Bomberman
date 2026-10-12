#![deny(unsafe_code)]
#![allow(clippy::module_inception)]

mod app;
mod app_state;
mod audio;
mod game;
mod graphics;
mod input;
mod settings;
mod ui;

use crate::settings::settings::Settings;
use app::App;
use game::resources::Resources;
use std::sync::OnceLock;
use winit::event_loop::{ControlFlow, EventLoop};

static RESOURCES: OnceLock<Resources> = OnceLock::new();

pub fn resources() -> &'static Resources {
    RESOURCES.get().expect("resources are not loaded yet")
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Bomberman: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::init(Settings::load_settings(), &event_loop, &RESOURCES)?;
    event_loop.run_app(&mut app)?;

    Ok(())
}
