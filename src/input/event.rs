use gilrs::{EventType, GamepadId};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use winit::keyboard::{KeyCode, PhysicalKey};

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
}

impl Display for InputEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Keyboard { key, down: _ } => {
                    if *key == PhysicalKey::Code(KeyCode::F35) {
                        UNBOUND_NAME.to_string()
                    } else {
                        let ret = format!("{:?}", key);
                        let ret = ret.split_at("Code(".len()).1;
                        pretty_key(ret.split_at(ret.len() - 1).0)
                    }
                }
                Self::ControllerInput { controller, event } => {
                    format!(
                        "{}:{}",
                        {
                            let ret = format!("{:?}", controller);
                            let ret = ret.split_at("GamepadId(".len()).1;
                            ret.split_at(ret.len() - 1).0.to_string()
                        },
                        match event {
                            EventType::ButtonChanged(button, _, _) => format!("{:?}", button),
                            EventType::AxisChanged(axis, value, _) => {
                                format!("{:#?}{}", axis, if *value < 0.0 { "-" } else { "+" })
                                    .replace("LeftStick", "LS ")
                                    .replace("RightStick", "RS ")
                            }
                            _ => "Invalid".to_string(),
                        }
                    )
                }
            }
        )
    }
}

pub const UNBOUND_NAME: &str = "Aucune";

fn pretty_key(code: &str) -> String {
    let code = code
        .strip_prefix("Key")
        .or_else(|| code.strip_prefix("Digit"))
        .unwrap_or(code);
    match code {
        "ArrowUp" => "Haut".into(),
        "ArrowDown" => "Bas".into(),
        "ArrowLeft" => "Gauche".into(),
        "ArrowRight" => "Droite".into(),
        "ShiftLeft" => "Maj gauche".into(),
        "ShiftRight" => "Maj droite".into(),
        "ControlLeft" => "Ctrl gauche".into(),
        "ControlRight" => "Ctrl droit".into(),
        "AltLeft" => "Alt gauche".into(),
        "AltRight" => "Alt droit".into(),
        "Space" => "Espace".into(),
        "Enter" => "Entrée".into(),
        "Escape" => "Échap".into(),
        "Backspace" => "Retour arrière".into(),
        "Tab" => "Tab".into(),
        _ => code.replace("Numpad", "Pavé "),
    }
}

impl InputEvent {
    pub fn unbound() -> Self {
        Self::Keyboard {
            key: PhysicalKey::Code(KeyCode::F35),
            down: true,
        }
    }
    pub fn from_keycode(code: KeyCode) -> Self {
        Self::Keyboard {
            key: PhysicalKey::Code(code),
            down: true,
        }
    }
}
