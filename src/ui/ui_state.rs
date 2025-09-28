use crate::{
    app_state::{AppState, KeyMap},
    audio::AudioManager,
    game::resources::Resources,
    graphics::StateRenderInfo,
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{button::Button, canvas::Canvas, game_settings::UIGameSettings},
};

#[derive(Debug, Copy, Clone)]
pub enum UIPage {
    MainMenu,
    Pause,
    GameSettings(UIGameSettings),
    GameOver,
    StageClear {
        timer: f32,
        next_level: u32,
        lives: u32,
    },
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub canvases: Vec<Canvas>,
    pub buttons: Vec<Button>,
    pub is_transparent: bool,
    pub selected: usize,
    pub page: UIPage,
    pub render_info: StateRenderInfo,
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

    pub fn tick(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        _keys: &KeyMap,
        resources: &Resources,
        audio_manager: &mut AudioManager,
    ) -> (Option<AppState>, u8) {
        match self.page {
            UIPage::MainMenu => self.main_menu_tick(inputs, audio_manager),
            UIPage::Pause => self.pause_tick(inputs, resources, audio_manager),
            UIPage::GameSettings(_) => self.game_settings_tick(delta, inputs, resources),
            UIPage::GameOver => self.game_over_tick(inputs, audio_manager),
            UIPage::StageClear { .. } => self.stage_clear_tick(delta),
        }
    }
}
