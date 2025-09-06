use std::{collections::HashMap, sync::Arc};

use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use winit::{event::ElementState, keyboard::PhysicalKey};

use crate::{
    audio::AudioManager,
    game::{arena_state::ArenaState, campaign_state::CampaignState, resources::Resources},
    graphics::{Renderer, Vulkan},
    input::input::Input,
    ui::UiState,
};

pub type KeyMap = HashMap<PhysicalKey, ElementState>;

#[derive(Debug, Clone)]
pub enum AppState {
    Arena(ArenaState),
    Campaign(CampaignState),
    Ui(UiState),
}

impl AppState {
    pub fn render(
        &self,
        renderer: &Renderer,
        vulkan: &Vulkan,
        resources: &Resources,
    ) -> Arc<SecondaryAutoCommandBuffer> {
        match self {
            AppState::Arena(arena_state) => arena_state.render(vulkan, renderer, resources),
            AppState::Campaign(campaign_state) => {
                campaign_state.render(vulkan, renderer, resources)
            }
            AppState::Ui(ui_state) => ui_state.render(vulkan, renderer, resources),
        }
    }

    pub fn tick(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        match self {
            AppState::Arena(arena_state) => {
                arena_state.tick(delta, inputs, keys, resources, audio_manager)
            }
            AppState::Campaign(campaign_state) => {
                campaign_state.tick(delta, inputs, keys, resources, audio_manager)
            }
            AppState::Ui(ui_state) => ui_state.tick(delta, inputs, keys, resources, audio_manager),
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self {
            AppState::Arena(_) | AppState::Campaign(_) => false,
            AppState::Ui(state) => state.is_transparent(),
        }
    }
}
