use std::fmt::Display;

use gilrs::{EventType, GamepadId};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use winit::{
    event::MouseButton,
    keyboard::{KeyCode, PhysicalKey},
};

/// The app merges inputs from any source into this structure
/// Anything that doesn't match is discarded
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputEvent {
    Keyboard {
        key: PhysicalKey,
        down: bool,
    },
    ControllerInput {
        controller: GamepadId,
        event: EventType,
    },
    Click {
        location: Vec2,
        button: MouseButton,
    },
}

impl Display for InputEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Keyboard { key, down: _ } => {
                    if *key == PhysicalKey::Code(KeyCode::F35) {
                        "Unbound".to_string()
                    } else {
                        let ret = format!("{:?}", key);
                        let ret = ret.split_at("Code(".len()).1;
                        ret.split_at(ret.len() - 1).0.to_string()
                    }
                }
                Self::ControllerInput { controller, event } => {
                    let ctr = {
                        let ret = format!("{:?}", controller);
                        let ret = ret.split_at("GamepadId(".len()).1;
                        ret.split_at(ret.len() - 1).0.to_string()
                    };
                    let btn = match event {
                        EventType::ButtonChanged(button, _, _) => format!("{:?}", button),
                        EventType::AxisChanged(axis, value, _) => {
                            let dir = if *value < 0.0 { "-" } else { "+" };
                            format!("{:#?}{dir}", axis)
                                .replace("LeftStick", "LS ")
                                .replace("RightStick", "RS ")
                        }
                        _ => "Invalid".to_string(),
                    };
                    format!("{ctr}:{btn}")
                }
                Self::Click {
                    location: _,
                    button,
                } => {
                    format!("{:?}", button)
                }
            }
        )
    }
}

impl InputEvent {
    /// Special value: KeyCode F35 is treated as unset
    pub fn unbound() -> Self {
        Self::Keyboard {
            key: PhysicalKey::Code(KeyCode::F35),
            down: true,
        }
    }
    /// Construct an InputEvent from a keyboard press
    pub fn from_keycode(code: KeyCode) -> Self {
        Self::Keyboard {
            key: PhysicalKey::Code(code),
            down: true,
        }
    }
}
