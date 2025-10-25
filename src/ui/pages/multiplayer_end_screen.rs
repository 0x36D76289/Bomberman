use glam::Vec2;

use crate::{
    app_state::AppState,
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{
        UiState,
        canvas::Canvas,
        consts::{SELECTED_TEXT_COLOR, TEXT_SIZE, WIN_BACKGROUND_COLOR},
    },
};

/// The space between the 2 lines of text
const LINE_SPACING: f32 = 0.1;

impl UiState {
    /// The multiplayer end screen ui page constructor
    pub fn multiplayer_end_screen(winners: Vec<u32>) -> Self {
        let text = if winners.len() == 1 {
            (
                "The winner is:".to_string(),
                format!("Player {}", winners[0] + 1),
            )
        } else {
            let mut ret = "Players ".to_string();
            let mut win_copy = winners.clone();

            while win_copy.len() >= 3 {
                ret += &format!("{}, ", win_copy.pop().unwrap() + 1);
            }
            if win_copy.len() == 2 {
                ret += &format!("{}, and ", win_copy.pop().unwrap() + 1);
            }
            if win_copy.len() == 1 {
                ret += &format!("{}", win_copy.pop().unwrap() + 1)
            }
            ("The winners are:".to_string(), ret)
        };
        Self {
            canvases: vec![
                Canvas {
                    center: Vec2 { x: 0.0, y: 0.0 },
                    width: 2.0,
                    height: 2.0,
                    color: WIN_BACKGROUND_COLOR,
                    texture: None,
                    text: None,
                    text_color: None,
                    text_size: None,
                },
                Canvas {
                    center: Vec2 {
                        x: 0.0,
                        y: -LINE_SPACING,
                    },
                    width: 0.0,
                    height: 0.0,
                    color: WIN_BACKGROUND_COLOR,
                    texture: None,
                    text: Some(text.0),
                    text_color: Some(SELECTED_TEXT_COLOR),
                    text_size: Some(TEXT_SIZE * 2.0),
                },
                Canvas {
                    center: Vec2 {
                        x: 0.0,
                        y: LINE_SPACING,
                    },
                    width: 0.0,
                    height: 0.0,
                    color: WIN_BACKGROUND_COLOR,
                    texture: None,
                    text: Some(text.1),
                    text_color: Some(SELECTED_TEXT_COLOR),
                    text_size: Some(TEXT_SIZE * 2.0),
                },
            ],
            buttons: vec![],
            selected: 0,
            render_info: Default::default(),
        }
    }

    /// The multiplayer end screen ui tick function
    pub fn multiplayer_end_screen_tick(&self, inputs: &Vec<Input>) -> (Option<AppState>, u8) {
        if inputs.menu_confirm() == InputState::Pressed {
            (None, 1)
        } else {
            (None, 0)
        }
    }
}
