use crate::game::state::State;
use crate::graphics::systems::point_light_system::{self, PointLightSystem};
use crate::graphics::systems::GlobalUbo;
use crate::graphics::systems::game_object_system::{self, GameObjectSystem};
use crate::graphics::{
    self, Camera, GameObject, Graphics, Light, Model, RenderContext, Renderer, Transform, Vulkan
};
use crate::input::{InputState, KeyboardMovementController};
use crate::load_model;
use glam::{Vec3, Vec4};
use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::KeyCode,
    window::WindowId,
};

pub struct App {
    pub state: State,
    pub graphics: Graphics
}

impl App {
    pub fn init(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        let graphics = Graphics::new(event_loop)?;

        let state = State::default_state(graphics.vulkan.memory_allocator.clone())?;

        Ok(Self {
            state,
            graphics
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let graphics = &mut self.graphics;

        graphics.renderer.init_render_context(event_loop, &graphics.vulkan);
        graphics.game_object_system.create_pipeline(
            &graphics.vulkan,
            graphics.renderer.get_rcx().render_pass.clone(),
            graphics.renderer.get_rcx().window.inner_size()
        );
        graphics.point_light_system.create_pipeline(
            &graphics.vulkan,
            graphics.renderer.get_rcx().render_pass.clone(),
            graphics.renderer.get_rcx().window.inner_size()
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // TODO: main loop
        // TODO: render

        // Event Handling
        match event {
            WindowEvent::RedrawRequested => {
                let state = &mut self.state;
                let renderer = &mut self.graphics.renderer;
                let game_object_system = &self.graphics.game_object_system;
                let point_light_system = &self.graphics.point_light_system;

                state.camera_controller.move_in_plane_xz(
                    &state.input_state,
                    renderer.get_delta_time(),
                    &mut state.viewer_object,
                );
                state.camera.set_view_xyz(
                    state.viewer_object.transform.translation,
                    state.viewer_object.transform.rotation,
                );
                state.camera.set_perspective_projection(
                    0.872664626,
                    renderer.get_aspect_ration(),
                    0.1,
                    100.0,
                );

                if let Some(mut command_buffer) = renderer.begin_frame(&self.graphics.vulkan) {
                    let global_ubo = GlobalUbo {
                        projection: state.camera.projection_matrix.to_cols_array_2d(),
                        view: state.camera.view_matrix.to_cols_array_2d(),
                        inverse_view: state.camera.inverse_view_matrix.to_cols_array_2d(),
                        ambient_light_color: [1.0, 1.0, 1.0, 0.2],
                        light_position: state.light.transform.translation.to_array().into(),
                        light_color: state.light.color.to_array(),
                    };

                    game_object_system.render_game_objects(&self.graphics.vulkan, &state.objects, global_ubo, &mut command_buffer);
                    point_light_system.render(&self.graphics.vulkan, global_ubo, &mut command_buffer);
                    point_light_system.update(&mut state.light, renderer.get_delta_time());
                    renderer.end_frame(&self.graphics.vulkan, command_buffer);
                    renderer.update_time();
                    renderer.update_title();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.state.input_state.update_keyboard_input(event)
            }
            WindowEvent::Resized(_) => self.graphics.renderer.recreate_swapchain(true),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics.renderer.request_redraw();
    }
}
