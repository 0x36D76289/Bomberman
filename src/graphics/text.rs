use crate::graphics::object::TextureIndex;
use anyhow::{Result, anyhow};
use glam::Vec2;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontId {
    Title,
    Body,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Glyph {
    pub uv: [f32; 4],
    pub size: Vec2,
    pub offset: Vec2,
    pub advance: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Font {
    pub texture: TextureIndex,
    pub px: f32,
    pub factor: f32,
    pub grid: f32,
    pub cap: f32,
    pub ascent: f32,
    pub line_height: f32,
    pub glyphs: HashMap<char, Glyph>,
}

pub struct FontAtlas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub font: Font,
}

fn charset() -> impl Iterator<Item = char> {
    (32u32..127)
        .chain(160..256)
        .chain([0x152, 0x153, 0x2019])
        .filter_map(char::from_u32)
}

pub fn build_font_atlas(ttf: &[u8], px: f32, factor: f32, grid: f32) -> Result<FontAtlas> {
    let font = fontdue::Font::from_bytes(ttf, fontdue::FontSettings::default())
        .map_err(|e| anyhow!("invalid font: {e}"))?;
    let line = font
        .horizontal_line_metrics(px)
        .ok_or_else(|| anyhow!("font has no horizontal metrics"))?;

    const WIDTH: u32 = 1024;
    const PADDING: u32 = 3;
    let rasterized: Vec<(char, fontdue::Metrics, Vec<u8>)> = charset()
        .map(|c| {
            let (metrics, bitmap) = font.rasterize(c, px);
            (c, metrics, bitmap)
        })
        .collect();

    let mut positions = Vec::with_capacity(rasterized.len());
    let (mut x, mut y, mut row_height) = (PADDING, PADDING, 0);
    for (_, metrics, _) in &rasterized {
        let (w, h) = (metrics.width as u32, metrics.height as u32);
        if x + w + PADDING > WIDTH {
            x = PADDING;
            y += row_height + PADDING;
            row_height = 0;
        }
        positions.push((x, y));
        x += w + PADDING;
        row_height = row_height.max(h);
    }
    let height = (y + row_height + PADDING).next_power_of_two();

    let mut pixels = vec![0u8; (WIDTH * height * 4) as usize];
    let mut glyphs = HashMap::new();
    for ((c, metrics, bitmap), (gx, gy)) in rasterized.iter().zip(positions) {
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let i = (((gy + row as u32) * WIDTH + gx + col as u32) * 4) as usize;
                pixels[i..i + 3].fill(255);
                pixels[i + 3] = if bitmap[row * metrics.width + col] >= 128 {
                    255
                } else {
                    0
                };
            }
        }
        let (w, h) = (metrics.width as f32, metrics.height as f32);
        glyphs.insert(
            *c,
            Glyph {
                uv: [
                    gx as f32 / WIDTH as f32,
                    gy as f32 / height as f32,
                    (gx as f32 + w) / WIDTH as f32,
                    (gy as f32 + h) / height as f32,
                ],
                size: Vec2::new(w, h),
                offset: Vec2::new(metrics.xmin as f32, -(h + metrics.ymin as f32)),
                advance: metrics.advance_width,
            },
        );
    }

    Ok(FontAtlas {
        width: WIDTH,
        height,
        pixels,
        font: Font {
            texture: -1,
            px,
            factor,
            grid,
            cap: glyphs.get(&'H').map_or(px * 0.7, |g: &Glyph| g.size.y),
            ascent: line.ascent,
            line_height: line.new_line_size,
            glyphs,
        },
    })
}

impl Font {
    pub fn glyph(&self, c: char) -> Option<&Glyph> {
        self.glyphs.get(&c).or_else(|| self.glyphs.get(&'?'))
    }

    pub fn em(&self, size: f32) -> f32 {
        let em = size * self.factor;
        if self.grid > 0.0 {
            (em / self.grid).round().max(1.0) * self.grid
        } else {
            em
        }
    }

    pub fn scale(&self, size: f32) -> f32 {
        self.em(size) / self.px
    }

    pub fn line_width(&self, text: &str, size: f32) -> f32 {
        text.chars()
            .filter_map(|c| self.glyph(c))
            .map(|g| g.advance * self.scale(size))
            .sum()
    }

    pub fn wrap(&self, text: &str, size: f32, max_width: f32) -> Vec<String> {
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            let mut line = String::new();
            for word in paragraph.split(' ') {
                let candidate = if line.is_empty() {
                    word.to_string()
                } else {
                    format!("{line} {word}")
                };
                if !line.is_empty() && self.line_width(&candidate, size) > max_width {
                    lines.push(std::mem::take(&mut line));
                    line = word.to_string();
                } else {
                    line = candidate;
                }
            }
            lines.push(line);
        }
        lines
    }
}
