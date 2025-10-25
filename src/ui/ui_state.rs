use crate::{
    graphics::StateRenderInfo,
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{button::Button, canvas::Canvas},
};

/// A [UiState] represents all UI Elements and their interactions
#[derive(Debug, Clone)]
pub struct UiState {
    pub canvases: Vec<Canvas>,
    pub buttons: Vec<Button>,
    pub selected: usize,
    pub render_info: StateRenderInfo,
}

impl UiState {
    /// Unselects the current button and selects the target
    fn select_button(&mut self, target: usize) {
        let mut target = target;
        if target >= self.buttons.len() {
            target = 0;
        }

        self.buttons[self.selected].toggle();
        self.buttons[target].toggle();
        self.selected = target;
    }

    /// Moves the selection to the appropriate button based on the neighbor property and inputs
    /// Returns true if confirm button is used
    pub fn button_inputs(&mut self, inputs: &Vec<Input>) -> bool {
        if !self.buttons.is_empty() {
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
}
