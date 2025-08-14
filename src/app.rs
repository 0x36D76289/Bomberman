use crate::game::state::State;
use crate::graphics::{GlobalUbo, Graphics, PointLight};
use core::f32;
use glam::{Vec3, Vec4};
use std::error::Error;
use winit::keyboard::KeyCode;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

pub struct App {
    pub state: State,
    pub graphics: Graphics,
}

impl App {
    pub fn init(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let graphics = Graphics::new(event_loop)?;

        let state = State::default_state(&graphics)?;

        Ok(Self { state, graphics })
    }

    fn update_world(&mut self) {
        let state = &mut self.state;

        // state.camera.set_view_xyz(
        //     state.entities[0].physics.unwrap().transform.translation,
        //     state.entities[0].physics.unwrap().transform.rotation,
        // );
        let map_center = Vec3::new(
            state.map.width as f32 / 2.0,
            0.0,
            state.map.height as f32 / 2.0,
        );
        state
            .camera
            .set_view_target(Vec3::new(map_center.x, -19.0, -10.0), map_center);
        // state.camera.set_view_xyz(Vec3::new(0.0, -19.0, -9.0), Vec3::new(-1.17, 0.0, 0.0));
        state.camera.set_perspective_projection(
            0.6,
            self.graphics.renderer.get_aspect_ratio(),
            0.1,
            1000.0,
        );
    }

    fn draw_frame(&mut self) {
        let state = &self.state;
        let renderer = &mut self.graphics.renderer;
        let game_object_system = &self.graphics.game_object_system;

        if let Some(mut command_buffer) = renderer.begin_frame(&self.graphics.vulkan) {
            let global_ubo = GlobalUbo {
                projection: state.camera.projection_matrix.to_cols_array_2d(),
                view: state.camera.view_matrix.to_cols_array_2d(),
                inverse_view: state.camera.inverse_view_matrix.to_cols_array_2d(),
                ambient_light_color: [1.0, 1.0, 1.0, 0.8],
                lights: [PointLight {
                    position: Vec4::ONE.to_array(),
                    color: Vec4::ONE.to_array(),
                }; 100],
                light_number: 0,
            };

            game_object_system.render_game_objects(
                &self.graphics.vulkan,
                &self.state,
                global_ubo,
                &mut command_buffer,
            );
            renderer.end_frame(&self.graphics.vulkan, command_buffer);
            renderer.update_time();
            renderer.update_title(&format!(
                "Bomberman!! fps: {:.0}",
                renderer.get_rcx().time_info.avg_fps,
            ));
        }
    }
}

impl ApplicationHandler for App {
    // This is called when the window is created
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let graphics = &mut self.graphics;

        graphics
            .renderer
            .init_render_context(event_loop, &graphics.vulkan);
        graphics.game_object_system.create_pipeline(
            &graphics.vulkan,
            graphics.renderer.get_rcx().render_pass.clone(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.state.tick(self.graphics.renderer.get_delta_time());
                self.update_world();
                self.draw_frame();
            }
            WindowEvent::Resized(_) => self.graphics.renderer.recreate_swapchain(true),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                self.state.record_key(event.physical_key, event.state);
                #[cfg(debug_assertions)]
                if event.state.is_pressed() && event.repeat == false {
                    if event.physical_key == KeyCode::Space {
                        self.state.print();
                    }
                    if event.physical_key == KeyCode::Escape {
                        event_loop.exit();
                    }
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.renderer.request_redraw();
    }
}
