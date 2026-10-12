use crate::ui::style;
use glam::Vec4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tint {
    Yellow,
    Blue,
    Green,
    Red,
    Grey,
}

impl Tint {
    pub fn color(self) -> Vec4 {
        match self {
            Tint::Yellow => style::YELLOW,
            Tint::Blue => style::rgb(0x00, 0x58, 0xf8),
            Tint::Green => style::GREEN,
            Tint::Red => style::RED,
            Tint::Grey => style::GREY,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoxColors {
    pub fill: Vec4,
    pub border: Vec4,
    pub light: Vec4,
    pub shade: Vec4,
}

impl BoxColors {
    fn bevel(color: Vec4) -> Self {
        Self {
            fill: color,
            border: style::INK,
            light: color.lerp(Vec4::ONE, 0.4),
            shade: (color.truncate() * 0.6).extend(1.0),
        }
    }

    fn flat(fill: Vec4, border: Vec4) -> Self {
        Self {
            fill,
            border,
            light: fill,
            shade: fill,
        }
    }
}

const STAR: [&str; 8] = [
    "...oo...", "..o##o..", "oo+###oo", "o######o", ".o####o.", ".o#oo#o.", "o#o..o#o", "oo....oo",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Singleplayer,
    Multiplayer,
    Gear,
    Trophy,
    Power,
    Home,
    Return,
    Music,
    Gamepad,
    Locked,
    Trashcan,
    Plus,
    Checkmark,
    Cross,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prompt {
    KeySpace,
    KeyEscape,
    KeyArrows,
    MouseLeft,
    MouseScroll,
    PadA,
    PadB,
    PadDpad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiTex {
    Button(Tint),
    Square(Tint),
    Panel,
    PanelGrey,
    CheckOn,
    CheckOff,
    ArrowLeft,
    ArrowRight,
    Star,
    StarEmpty,
    Divider,
    Icon(Icon),
    Prompt(Prompt),
}

impl UiTex {
    pub fn files() -> Vec<(UiTex, String)> {
        let mut files = Vec::new();
        for (icon, name) in [
            (Icon::Singleplayer, "singleplayer"),
            (Icon::Multiplayer, "multiplayer"),
            (Icon::Gear, "gear"),
            (Icon::Trophy, "trophy"),
            (Icon::Power, "power"),
            (Icon::Home, "home"),
            (Icon::Return, "return"),
            (Icon::Music, "musicOn"),
            (Icon::Gamepad, "gamepad"),
            (Icon::Locked, "locked"),
            (Icon::Trashcan, "trashcan"),
            (Icon::Plus, "plus"),
            (Icon::Checkmark, "checkmark"),
            (Icon::Cross, "cross"),
            (Icon::Next, "next"),
        ] {
            files.push((UiTex::Icon(icon), format!("ui/icons/{name}.png")));
        }
        for (prompt, name) in [
            (Prompt::KeySpace, "keyboard_space"),
            (Prompt::KeyEscape, "keyboard_escape"),
            (Prompt::KeyArrows, "keyboard_arrows"),
            (Prompt::MouseLeft, "mouse_left"),
            (Prompt::MouseScroll, "mouse_scroll"),
            (Prompt::PadA, "xbox_button_a"),
            (Prompt::PadB, "xbox_button_b"),
            (Prompt::PadDpad, "xbox_dpad"),
        ] {
            files.push((UiTex::Prompt(prompt), format!("ui/prompts/{name}.png")));
        }
        files
    }

    pub fn box_colors(self) -> Option<BoxColors> {
        Some(match self {
            UiTex::Button(tint) | UiTex::Square(tint) => BoxColors::bevel(tint.color()),
            UiTex::Panel => BoxColors::flat(style::rgb(0x0c, 0x0c, 0x2c), style::WHITE),
            UiTex::PanelGrey => BoxColors::flat(style::PANEL_GREY, style::TEXT_MUTED),
            _ => return None,
        })
    }

    pub fn sprite(self) -> Option<(&'static [&'static str], Vec4)> {
        Some(match self {
            UiTex::Star => (&STAR, style::YELLOW),
            UiTex::StarEmpty => (&STAR, style::GREY),
            UiTex::CheckOn => (
                &[
                    "ooooooooo",
                    "o#######o",
                    "o######+o",
                    "o#####++o",
                    "o+###++#o",
                    "o++#++##o",
                    "o#+++###o",
                    "o##+####o",
                    "ooooooooo",
                ],
                style::GREEN,
            ),
            UiTex::CheckOff => (
                &[
                    "ooooooooo",
                    "o#######o",
                    "o#######o",
                    "o#######o",
                    "o#######o",
                    "o#######o",
                    "o#######o",
                    "o#######o",
                    "ooooooooo",
                ],
                style::PANEL_GREY,
            ),
            UiTex::ArrowLeft => (
                &[
                    "....#", "...##", "..###", ".####", "..###", "...##", "....#",
                ],
                style::WHITE,
            ),
            UiTex::ArrowRight => (
                &[
                    "#....", "##...", "###..", "####.", "###..", "##...", "#....",
                ],
                style::WHITE,
            ),
            UiTex::Divider => (&["#"], style::WHITE),
            _ => return None,
        })
    }
}
