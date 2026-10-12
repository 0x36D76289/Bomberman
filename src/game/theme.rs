use crate::{
    audio::BackgroundMusic,
    game::resources::{DecorKind, ResourceName},
    graphics::LightInfo,
};
use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Theme {
    #[default]
    Grass,
    Desert,
    Snow,
    Castle,
    Volcano,
    Arena,
}

impl Theme {
    pub const ALL: [Theme; 6] = [
        Theme::Grass,
        Theme::Desert,
        Theme::Snow,
        Theme::Castle,
        Theme::Volcano,
        Theme::Arena,
    ];

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|theme| theme.file_name().eq_ignore_ascii_case(name.trim()))
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Theme::Grass => "grass",
            Theme::Desert => "desert",
            Theme::Snow => "snow",
            Theme::Castle => "castle",
            Theme::Volcano => "volcano",
            Theme::Arena => "arena",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Theme::Grass => "Vertes Prairies",
            Theme::Desert => "Désert des Dunes",
            Theme::Snow => "Pics Gelés",
            Theme::Castle => "Vieux Château",
            Theme::Volcano => "Cœur du Volcan",
            Theme::Arena => "Arène de Combat",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Theme::Grass => "Prairies",
            Theme::Desert => "Désert",
            Theme::Snow => "Neige",
            Theme::Castle => "Château",
            Theme::Volcano => "Volcan",
            Theme::Arena => "Arène",
        }
    }

    pub fn ui_color(self) -> Vec4 {
        match self {
            Theme::Grass => Vec4::new(0.0, 0.66, 0.0, 1.0),
            Theme::Desert => Vec4::new(0.97, 0.47, 0.35, 1.0),
            Theme::Snow => Vec4::new(0.24, 0.74, 0.99, 1.0),
            Theme::Castle => Vec4::new(0.47, 0.47, 0.47, 1.0),
            Theme::Volcano => Vec4::new(0.85, 0.16, 0.0, 1.0),
            Theme::Arena => Vec4::new(0.97, 0.72, 0.0, 1.0),
        }
    }

    pub fn block_model(self) -> ResourceName {
        match self {
            Theme::Grass => ResourceName::BlockGrass,
            Theme::Snow | Theme::Castle => ResourceName::BlockSnow,
            Theme::Desert | Theme::Volcano | Theme::Arena => ResourceName::Brick,
        }
    }

    pub fn block_tint(self) -> Vec3 {
        match self {
            Theme::Desert => Vec3::new(1.0, 0.85, 0.6),
            Theme::Castle => Vec3::new(0.55, 0.56, 0.62),
            Theme::Volcano => Vec3::new(0.55, 0.35, 0.3),
            Theme::Arena => Vec3::new(0.55, 0.6, 0.8),
            _ => Vec3::ONE,
        }
    }

    pub fn floor_tint(self) -> Vec3 {
        match self {
            Theme::Arena => Vec3::new(0.75, 0.75, 0.8),
            _ => Vec3::ONE,
        }
    }

    pub fn crate_tint(self) -> Vec3 {
        match self {
            Theme::Snow => Vec3::new(0.85, 0.95, 1.1),
            Theme::Volcano => Vec3::new(1.0, 0.6, 0.45),
            Theme::Castle => Vec3::new(0.85, 0.8, 0.75),
            _ => Vec3::ONE,
        }
    }

    pub fn decor(self) -> &'static [DecorKind] {
        match self {
            Theme::Grass => &[
                DecorKind::Tree,
                DecorKind::TreePine,
                DecorKind::Flowers,
                DecorKind::Grass,
                DecorKind::Mushrooms,
                DecorKind::Rocks,
            ],
            Theme::Desert => &[DecorKind::Rocks, DecorKind::Stones, DecorKind::Grass],
            Theme::Snow => &[
                DecorKind::TreeSnow,
                DecorKind::TreePineSnow,
                DecorKind::Rocks,
            ],
            Theme::Castle => &[DecorKind::Fence, DecorKind::Stones, DecorKind::TreePine],
            Theme::Volcano => &[DecorKind::Rocks, DecorKind::Stones],
            Theme::Arena => &[DecorKind::Fence, DecorKind::Flowers, DecorKind::Tree],
        }
    }

    pub fn decor_tint(self) -> Vec3 {
        match self {
            Theme::Volcano => Vec3::new(0.5, 0.35, 0.3),
            Theme::Desert => Vec3::new(1.0, 0.9, 0.7),
            _ => Vec3::ONE,
        }
    }

    pub fn light(self) -> LightInfo {
        let (ambient, directional) = match self {
            Theme::Volcano => (
                Vec4::new(1.0, 0.55, 0.45, 0.5),
                Vec4::new(1.0, 0.6, 0.35, 0.75),
            ),
            Theme::Snow => (Vec4::new(0.75, 0.85, 1.0, 0.6), Vec4::ONE.with_w(0.65)),
            Theme::Desert => (Vec4::new(1.0, 0.9, 0.75, 0.6), Vec4::ONE.with_w(0.7)),
            Theme::Castle => (Vec4::new(0.7, 0.7, 1.0, 0.5), Vec4::ONE.with_w(0.7)),
            _ => (Vec4::ONE.with_w(0.55), Vec4::ONE.with_w(0.7)),
        };
        LightInfo {
            ambient_light_color: ambient,
            direction_to_light: Vec3::new(0.0, -3.0, 1.0).normalize(),
            directional_light_color: directional,
        }
    }

    pub fn music(self) -> BackgroundMusic {
        match self {
            Theme::Grass => BackgroundMusic::World1,
            Theme::Desert => BackgroundMusic::World2,
            Theme::Snow => BackgroundMusic::World3,
            Theme::Castle => BackgroundMusic::World4,
            Theme::Volcano => BackgroundMusic::Boss,
            Theme::Arena => BackgroundMusic::Arena,
        }
    }
}
