use crate::{
    game::{enemy::EnemyKind, theme::Theme},
    graphics::{
        Model, Vulkan,
        animation::Animation,
        mesh::{FbxImport, SkinnedMesh},
        object::{Object, TextureIndex},
        text::{self, Font, FontId},
        texture::load_texture_rgba,
        transform::Transform,
    },
    settings::assets::read_asset,
    ui::textures::UiTex,
};
use anyhow::{Context, Result};
use std::{collections::HashMap, fmt::Write, sync::Arc};
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract},
    image::view::ImageView,
    sync::GpuFuture,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecorKind {
    Tree,
    TreePine,
    TreeSnow,
    TreePineSnow,
    Rocks,
    Stones,
    Grass,
    Flowers,
    Mushrooms,
    Fence,
}

impl DecorKind {
    fn file(self) -> (&'static str, f32) {
        match self {
            DecorKind::Tree => ("tree", 1.1),
            DecorKind::TreePine => ("tree-pine", 1.0),
            DecorKind::TreeSnow => ("tree-snow", 1.1),
            DecorKind::TreePineSnow => ("tree-pine-snow", 1.0),
            DecorKind::Rocks => ("rocks", 0.8),
            DecorKind::Stones => ("stones", 0.7),
            DecorKind::Grass => ("grass", 0.6),
            DecorKind::Flowers => ("flowers", 0.8),
            DecorKind::Mushrooms => ("mushrooms", 0.6),
            DecorKind::Fence => ("fence-straight", 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceName {
    Player,
    Bomb,
    FlameCenter,
    FlameArm,
    FlameTip,
    Floor(Theme),
    Crate,
    BlockGrass,
    BlockSnow,
    Brick,
    Exit,
    PowerSpeed,
    PowerFire,
    PowerBomb,
    PowerKick,
    PowerRemote,
    PowerWallPass,
    PowerVest,
    PowerLife,
    PowerSkull,
    Enemy(EnemyKind),
    Decor(DecorKind),
}

#[derive(Debug, Clone)]
pub struct PlayerAnimations {
    pub idle: Animation,
    pub run: Animation,
    pub bomb: Animation,
}

#[derive(Debug, Clone)]
pub struct EnemyModel {
    pub mesh: SkinnedMesh,
    pub idle: Animation,
    pub walk: Animation,
}

#[derive(Debug, Clone)]
pub struct Resources {
    pub textures: Vec<Arc<ImageView>>,
    pub textures_index: HashMap<ResourceName, TextureIndex>,
    pub models: HashMap<ResourceName, Arc<Model>>,
    pub player_mesh: SkinnedMesh,
    pub player_animations: PlayerAnimations,
    pub enemies: HashMap<EnemyKind, EnemyModel>,
    pub ui: HashMap<UiTex, (TextureIndex, glam::Vec2)>,
    pub fonts: HashMap<FontId, Font>,
}

pub const FLOOR_SIZE: u32 = 160;
const FLOOR_PIXELS: u32 = 16;

impl Resources {
    pub fn object(&self, name: ResourceName, transform: Transform, color: glam::Vec3) -> Object {
        Object {
            model: self.models[&name].clone(),
            texture: Some(self.textures_index[&name]),
            transform,
            color,
        }
    }

    pub fn load_resources(vulkan: &Vulkan) -> Result<Self> {
        let allocator = vulkan.memory_allocator.clone();
        let mut textures = Vec::new();
        let mut textures_index = HashMap::new();
        let mut models = HashMap::new();

        let mut command_buffer = AutoCommandBufferBuilder::primary(
            vulkan.command_buffer_allocator.clone(),
            vulkan.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        let decode = |path: &str| {
            image::load_from_memory(&read_asset(path)?)
                .with_context(|| format!("invalid image {path}"))
        };
        let mut add_texture = |image: image::RgbaImage| {
            let (w, h) = image.dimensions();
            textures.push(load_texture_rgba(
                w,
                h,
                image.as_raw(),
                &mut command_buffer,
                allocator.clone(),
            ));
            (
                (textures.len() - 1) as TextureIndex,
                glam::Vec2::new(w as f32, h as f32),
            )
        };

        let player_texture = add_texture(decode("textures/player.png")?.into_rgba8()).0;
        let colormap = add_texture(decode("textures/colormap.png")?.into_rgba8()).0;
        for theme in Theme::ALL {
            textures_index.insert(
                ResourceName::Floor(theme),
                add_texture(
                    decode(&format!("textures/floor_{}.jpg", theme.file_name()))?
                        .resize_exact(
                            FLOOR_PIXELS,
                            FLOOR_PIXELS,
                            image::imageops::FilterType::Triangle,
                        )
                        .into_rgba8(),
                )
                .0,
            );
        }

        let mut ui = HashMap::new();
        for (tex, path) in UiTex::files() {
            ui.insert(tex, add_texture(decode(&path)?.into_rgba8()));
        }
        let mut fonts = HashMap::new();
        for (id, path, px, factor, grid) in [
            (FontId::Title, "fonts/title.ttf", 16.0, 0.6, 4.0),
            (FontId::Body, "fonts/body.ttf", 48.0, 1.0, 0.0),
        ] {
            let atlas = text::build_font_atlas(&read_asset(path)?, px, factor, grid)?;
            let mut font = atlas.font;
            font.texture = add_texture(
                image::RgbaImage::from_raw(atlas.width, atlas.height, atlas.pixels)
                    .with_context(|| format!("invalid font atlas {path}"))?,
            )
            .0;
            fonts.insert(id, font);
        }

        let load = |path: &str, normalize: Option<f32>| -> Result<Arc<Model>> {
            let bytes = read_asset(path)?;
            Model::load(
                &match normalize {
                    Some(width) => normalize_obj(&bytes, width),
                    None => bytes,
                },
                allocator.clone(),
            )
            .map_err(|e| anyhow::anyhow!("invalid model {path}: {e}"))
        };

        let fbx = |bytes: &[u8], path: &str| {
            FbxImport::from_bytes(bytes, allocator.clone())
                .map_err(|e| anyhow::anyhow!("invalid model {path}: {e}"))
        };
        let animation = |name: &str| {
            let path = format!("animations/{name}.fbx");
            fbx(&read_asset(&path)?, &path)?
                .animations
                .into_iter()
                .max_by_key(|animation| animation.length)
                .with_context(|| format!("{path} has no animation"))
        };
        textures_index.insert(ResourceName::Player, player_texture);

        let floor = Model::load(floor_obj(FLOOR_SIZE).as_bytes(), allocator.clone())
            .map_err(|e| anyhow::anyhow!("floor mesh: {e}"))?;
        for theme in Theme::ALL {
            models.insert(ResourceName::Floor(theme), floor.clone());
        }

        for (name, obj) in [
            (ResourceName::FlameCenter, flame_dome_obj()),
            (ResourceName::FlameArm, flame_tube_obj(false)),
            (ResourceName::FlameTip, flame_tube_obj(true)),
        ] {
            models.insert(
                name,
                Model::load(obj.as_bytes(), allocator.clone())
                    .map_err(|e| anyhow::anyhow!("flame mesh: {e}"))?,
            );
        }

        for (name, file, width) in [
            (ResourceName::Bomb, "bomb", 1.9),
            (ResourceName::Crate, "crate", 1.0),
            (ResourceName::BlockGrass, "block-grass", 1.0),
            (ResourceName::BlockSnow, "block-snow", 1.0),
            (ResourceName::Brick, "brick", 1.0),
            (ResourceName::Exit, "flag", 0.5),
            (ResourceName::PowerSpeed, "jewel", 0.5),
            (ResourceName::PowerFire, "star", 0.6),
            (ResourceName::PowerBomb, "bomb", 0.5),
            (ResourceName::PowerKick, "coin-gold", 0.55),
            (ResourceName::PowerRemote, "key", 0.6),
            (ResourceName::PowerWallPass, "chest", 0.55),
            (ResourceName::PowerVest, "mushrooms", 0.6),
            (ResourceName::PowerLife, "heart", 0.6),
            (ResourceName::PowerSkull, "rocks", 0.6),
        ] {
            models.insert(name, load(&format!("models/{file}.obj"), Some(width))?);
            textures_index.insert(name, colormap);
        }
        let mut enemies = HashMap::new();
        for kind in [
            EnemyKind::Balloom,
            EnemyKind::Onil,
            EnemyKind::Doll,
            EnemyKind::Minvo,
            EnemyKind::Boss,
        ] {
            let path = format!("models/{}.fbx", kind.model().0);
            let bytes = read_asset(&path)?;
            let import = fbx(&bytes, &path)?;
            let names = animation_names(&bytes)?;
            let animation = |name: &str| {
                names
                    .iter()
                    .position(|n| n == name)
                    .and_then(|i| import.animations.get(i).cloned())
                    .with_context(|| format!("{path} has no {name} animation"))
            };
            enemies.insert(
                kind,
                EnemyModel {
                    idle: animation("idle")?,
                    walk: animation("walk")?,
                    mesh: import
                        .skinned_meshes
                        .first()
                        .cloned()
                        .with_context(|| format!("{path} has no skinned mesh"))?,
                },
            );
            textures_index.insert(ResourceName::Enemy(kind), colormap);
        }
        for kind in [
            DecorKind::Tree,
            DecorKind::TreePine,
            DecorKind::TreeSnow,
            DecorKind::TreePineSnow,
            DecorKind::Rocks,
            DecorKind::Stones,
            DecorKind::Grass,
            DecorKind::Flowers,
            DecorKind::Mushrooms,
            DecorKind::Fence,
        ] {
            let (file, width) = kind.file();
            let name = ResourceName::Decor(kind);
            models.insert(name, load(&format!("models/{file}.obj"), Some(width))?);
            textures_index.insert(name, colormap);
        }

        command_buffer
            .build()?
            .execute(vulkan.queue.clone())?
            .then_signal_fence_and_flush()?
            .wait(None)?;

        Ok(Self {
            textures,
            textures_index,
            models,
            player_mesh: fbx(&read_asset("models/player.fbx")?, "models/player.fbx")?
                .skinned_meshes
                .into_iter()
                .next()
                .context("models/player.fbx has no skinned mesh")?,
            player_animations: PlayerAnimations {
                idle: animation("player_idle")?,
                run: animation("player_run")?,
                bomb: animation("player_bomb2")?,
            },
            enemies,
            ui,
            fonts,
        })
    }
}

fn animation_names(bytes: &[u8]) -> Result<Vec<String>> {
    Ok(ufbx::load_memory(bytes, ufbx::LoadOpts::default())
        .map_err(|e| anyhow::anyhow!("FBX error: {e:?}"))?
        .anim_stacks
        .iter()
        .map(|stack| stack.element.name.to_string())
        .collect())
}

fn normalize_obj(bytes: &[u8], width: f32) -> Vec<u8> {
    let text = String::from_utf8_lossy(bytes);
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    let vertex = |line: &str| -> Option<[f32; 3]> {
        let mut values = line.strip_prefix("v ")?.split_whitespace();
        let mut v = [0.0; 3];
        for value in &mut v {
            *value = values.next()?.parse().ok()?;
        }
        Some(v)
    };

    for v in text.lines().filter_map(vertex) {
        for axis in 0..3 {
            min[axis] = min[axis].min(v[axis]);
            max[axis] = max[axis].max(v[axis]);
        }
    }
    if min[0] > max[0] {
        return bytes.to_vec();
    }

    let scale = width / (max[0] - min[0]).max(max[2] - min[2]).max(f32::EPSILON);
    let center = [(min[0] + max[0]) / 2.0, min[1], (min[2] + max[2]) / 2.0];

    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        match vertex(line) {
            Some(v) => {
                let _ = writeln!(
                    out,
                    "v {} {} {}",
                    (v[0] - center[0]) * scale,
                    (v[1] - center[1]) * scale,
                    (v[2] - center[2]) * scale
                );
            }
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    out.into_bytes()
}

const FLAME_SEGMENTS: usize = 12;

fn surface_obj(
    out: &mut String,
    first: &mut usize,
    nu: usize,
    nv: usize,
    f: impl Fn(f32, f32) -> [f32; 3],
) {
    for j in 0..=nv {
        for i in 0..=nu {
            let [x, y, z] = f(i as f32 / nu as f32, j as f32 / nv as f32);
            let _ = writeln!(out, "v {x} {y} {z}");
        }
    }
    let index = |i: usize, j: usize| *first + j * (nu + 1) + i;
    for j in 0..nv {
        for i in 0..nu {
            let (a, b, c, d) = (
                index(i, j),
                index(i + 1, j),
                index(i + 1, j + 1),
                index(i, j + 1),
            );
            let _ = writeln!(out, "f {a} {b} {c}\nf {a} {c} {d}");
        }
    }
    *first += (nu + 1) * (nv + 1);
}

fn dome_point(around: f32, up: f32) -> [f32; 3] {
    [
        0.5 * up.cos() * around.cos(),
        0.5 * up.sin(),
        0.5 * up.cos() * around.sin(),
    ]
}

fn flame_dome_obj() -> String {
    use std::f32::consts::{FRAC_PI_2, TAU};
    let mut out = String::new();
    surface_obj(
        &mut out,
        &mut 1,
        FLAME_SEGMENTS,
        FLAME_SEGMENTS / 2,
        |u, v| dome_point(u * TAU, v * FRAC_PI_2),
    );
    out
}

fn flame_tube_obj(tip: bool) -> String {
    use std::f32::consts::{FRAC_PI_2, PI};
    let mut out = String::new();
    let mut first = 1;
    surface_obj(&mut out, &mut first, FLAME_SEGMENTS / 2, 1, |u, v| {
        let angle = u * PI;
        [
            -0.5 + v * (if tip { 0.0 } else { 0.5 } + 0.5),
            0.5 * angle.sin(),
            0.5 * angle.cos(),
        ]
    });
    if tip {
        surface_obj(
            &mut out,
            &mut first,
            FLAME_SEGMENTS / 2,
            FLAME_SEGMENTS / 2,
            |u, v| dome_point(-FRAC_PI_2 + u * PI, v * FRAC_PI_2),
        );
    }
    out
}

fn floor_obj(size: u32) -> String {
    let half = size as f32 / 2.0;
    let s = size as f32;
    format!(
        "v {n} 0 {n}\nv {p} 0 {n}\nv {p} 0 {p}\nv {n} 0 {p}\n\
         vt 0 0\nvt {s} 0\nvt {s} {s}\nvt 0 {s}\n\
         vn 0 1 0\n\
         f 1/1/1 3/3/1 2/2/1\nf 1/1/1 4/4/1 3/3/1\n",
        n = -half,
        p = half,
    )
}
