use crate::{
    graphics::StateRenderInfo,
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{button::Button, canvas::Canvas},
};

// #[derive(Debug, Copy, Clone)]
// pub enum UIPage {
//     MainMenu,
//     Pause,
//     GameSettings(UIGameSettings),
//     GameOver,
//     StageClear {
//         timer: f32,
//         next_level: u32,
//         lives: u32,
//     },
// }

#[derive(Debug, Clone)]
pub struct UiState {
    pub canvases: Vec<Canvas>,
    pub buttons: Vec<Button>,
    pub selected: usize,
    pub render_info: StateRenderInfo,
}

impl UiState {
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
