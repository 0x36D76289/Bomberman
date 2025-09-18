use crate::{
    app_state::{AppState, KeyMap},
    audio::AudioManager,
    game::resources::Resources,
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{button::Button, canvas::Canvas},
};

/// What UI is in use
#[derive(Debug, Copy, Clone)]
pub enum UIPage {
    MainMenu,
    Pause,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub canvases: Vec<Canvas>,
    pub buttons: Vec<Button>,
    pub is_transparent: bool,
    pub selected: usize,
    pub page: UIPage,
}

impl UiState {
    pub fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    fn select_button(&mut self, target: usize) {
        let mut target = target;
        if target >= self.buttons.len() {
            target = 0;
        }

        self.buttons[self.selected].toggle();
        self.buttons[target].toggle();
        self.selected = target;
    }

    /// Returns true if confirm button is used
    pub fn button_inputs(&mut self, inputs: &Vec<Input>) -> bool {
        if self.buttons.len() != 0 {
            if inputs.menu_up() == InputState::Pressed {
                self.select_button(self.buttons[self.selected].neighbors.up);
            }
            if inputs.menu_down() == InputState::Pressed {
                self.select_button(self.buttons[self.selected].neighbors.down);
            }
            if inputs.menu_left() == InputState::Pressed {
                self.select_button(self.buttons[self.selected].neighbors.left);
            }
            if inputs.menu_right() == InputState::Pressed {
                self.select_button(self.buttons[self.selected].neighbors.right);
            }
        }
        inputs.menu_confirm() == InputState::Pressed
    }

    pub fn tick(
        &mut self,
        _delta_time: f32,
        inputs: &Vec<Input>,
        keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        match self.page {
            UIPage::MainMenu => self.main_menu_tick(keys, resources, audio_manager),
            UIPage::Pause => self.pause_tick(inputs, resources, audio_manager),
        }
    }
}
