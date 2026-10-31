use crate::{
    game::resources::Resources,
    graphics::{
        UiVertex,
        text::{Font, FontId},
    },
    ui::{layout::Rect, style, textures::UiTex},
};
use glam::{Vec2, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub font: FontId,
    pub size: f32,
    pub color: Vec4,
    pub align: Align,
    pub outline: Option<(Vec4, f32)>,
    pub shadow: Option<(Vec4, Vec2)>,
}

impl TextStyle {
    pub fn new(font: FontId, size: f32, color: Vec4) -> Self {
        Self {
            font,
            size,
            color,
            align: Align::Center,
            outline: None,
            shadow: None,
        }
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn outline(mut self, color: Vec4, width: f32) -> Self {
        self.outline = Some((color, width));
        self
    }

    pub fn shadow(mut self, color: Vec4, offset: Vec2) -> Self {
        self.shadow = Some((color, offset));
        self
    }
}

pub struct DrawList {
    pub verts: Vec<UiVertex>,
    pub screen: Vec2,
    pub resources: &'static Resources,
}

impl DrawList {
    pub fn new(screen: Vec2, resources: &'static Resources) -> Self {
        Self {
            verts: Vec::with_capacity(4096),
            screen,
            resources,
        }
    }

    fn vertex(&mut self, p: Vec2, uv: Vec2, color: Vec4, tex: i32) {
        self.verts.push(UiVertex {
            position: [
                p.x / self.screen.x * 2.0 - 1.0,
                p.y / self.screen.y * 2.0 - 1.0,
            ],
            uv: uv.into(),
            color: color.into(),
            tex,
        });
    }

    fn shaded_quad(&mut self, rect: Rect, uv: [f32; 4], top: Vec4, bottom: Vec4, tex: i32) {
        let (l, t, r, b) = (rect.x, rect.y, rect.right(), rect.bottom());
        for i in [0, 1, 2, 2, 3, 0] {
            let (p, uv, color) = [
                (Vec2::new(l, t), Vec2::new(uv[0], uv[1]), top),
                (Vec2::new(r, t), Vec2::new(uv[2], uv[1]), top),
                (Vec2::new(r, b), Vec2::new(uv[2], uv[3]), bottom),
                (Vec2::new(l, b), Vec2::new(uv[0], uv[3]), bottom),
            ][i];
            self.vertex(p, uv, color, tex);
        }
    }

    pub fn quad(&mut self, rect: Rect, uv: [f32; 4], color: Vec4, tex: i32) {
        if rect.w <= 0.0 || rect.h <= 0.0 || color.w <= 0.0 {
            return;
        }
        self.shaded_quad(rect, uv, color, color, tex);
    }

    pub fn rect(&mut self, rect: Rect, color: Vec4) {
        self.quad(rect, [0.0, 0.0, 1.0, 1.0], color, -1);
    }

    pub fn gradient(&mut self, rect: Rect, top: Vec4, bottom: Vec4) {
        self.shaded_quad(rect, [0.0; 4], top, bottom, -1);
    }

    fn texture(&self, tex: UiTex) -> (i32, Vec2) {
        if let Some((rows, _)) = tex.sprite() {
            return (
                -1,
                Vec2::new(
                    rows.iter().map(|row| row.len()).max().unwrap_or(1) as f32,
                    rows.len() as f32,
                ),
            );
        }
        self.resources
            .ui
            .get(&tex)
            .copied()
            .unwrap_or((-1, Vec2::ONE))
    }

    pub fn image(&mut self, tex: UiTex, rect: Rect, color: Vec4) {
        if let Some((rows, main)) = tex.sprite() {
            self.sprite(rows, main, rect, color);
            return;
        }
        self.quad(rect, [0.0, 0.0, 1.0, 1.0], color, self.texture(tex).0);
    }

    pub fn image_fit(&mut self, tex: UiTex, rect: Rect, color: Vec4) {
        let (_, size) = self.texture(tex);
        self.image(
            tex,
            Rect::centered(rect.center(), size * (rect.w / size.x).min(rect.h / size.y)),
            color,
        );
    }

    pub fn frame(&mut self, tex: UiTex, rect: Rect, color: Vec4, border_scale: f32) {
        let Some(colors) = tex.box_colors() else {
            self.image(tex, rect, color);
            return;
        };
        let px = (border_scale * 8.0).round().max(2.0);
        let tint = |c: Vec4| c * color;

        self.notched(
            rect.translate(Vec2::splat(px)),
            px,
            style::fade(style::INK, 0.6 * color.w),
        );
        self.notched(rect, px, tint(colors.border));
        let inner = rect.inset(px);
        self.rect(inner, tint(colors.fill));
        let band = px.min(inner.h / 4.0);
        self.rect(
            Rect::new(inner.x, inner.y, inner.w, band),
            tint(colors.light),
        );
        self.rect(
            Rect::new(inner.x, inner.bottom() - band, inner.w, band),
            tint(colors.shade),
        );
    }

    pub fn outline(&mut self, r: Rect, px: f32, color: Vec4) {
        self.rect(Rect::new(r.x + px, r.y, r.w - 2.0 * px, px), color);
        self.rect(
            Rect::new(r.x + px, r.bottom() - px, r.w - 2.0 * px, px),
            color,
        );
        self.rect(Rect::new(r.x, r.y + px, px, r.h - 2.0 * px), color);
        self.rect(
            Rect::new(r.right() - px, r.y + px, px, r.h - 2.0 * px),
            color,
        );
    }

    fn notched(&mut self, r: Rect, px: f32, color: Vec4) {
        self.rect(Rect::new(r.x + px, r.y, r.w - 2.0 * px, r.h), color);
        self.rect(Rect::new(r.x, r.y + px, px, r.h - 2.0 * px), color);
        self.rect(
            Rect::new(r.right() - px, r.y + px, px, r.h - 2.0 * px),
            color,
        );
    }

    fn sprite(&mut self, rows: &[&str], main: Vec4, rect: Rect, color: Vec4) {
        let cell = Vec2::new(
            rect.w / rows.iter().map(|row| row.len()).max().unwrap_or(1) as f32,
            rect.h / rows.len() as f32,
        );
        for (y, row) in rows.iter().enumerate() {
            let mut cells = row.bytes().enumerate().peekable();
            while let Some((x, c)) = cells.next() {
                let fill = match c {
                    b'#' => main * color,
                    b'+' => style::WHITE * color,
                    b'o' => style::fade(style::INK, color.w),
                    _ => continue,
                };
                let mut len = 1;
                while cells.next_if(|(_, next)| *next == c).is_some() {
                    len += 1;
                }
                self.rect(
                    Rect::new(
                        rect.x + x as f32 * cell.x,
                        rect.y + y as f32 * cell.y,
                        len as f32 * cell.x,
                        cell.y,
                    ),
                    fill,
                );
            }
        }
    }

    pub fn font(&self, id: FontId) -> &'static Font {
        &self.resources.fonts[&id]
    }

    pub fn text_width(&self, text: &str, style: &TextStyle) -> f32 {
        self.font(style.font).line_width(text, style.size)
    }

    pub fn text(&mut self, text: &str, pos: Vec2, style: TextStyle) {
        if let Some((color, offset)) = style.shadow {
            self.text_layer(text, pos + offset, &style, color);
        }

        if let Some((color, width)) = style.outline {
            self.text_layer(
                text,
                pos + Vec2::splat(width.round().max(1.0)),
                &style,
                color,
            );
        }
        self.text_layer(text, pos, &style, style.color);
    }

    fn text_layer(&mut self, text: &str, pos: Vec2, style: &TextStyle, color: Vec4) {
        let font = self.font(style.font);
        let scale = font.scale(style.size);
        let width = font.line_width(text, style.size);
        let mut x = match style.align {
            Align::Left => pos.x,
            Align::Center => pos.x - width / 2.0,
            Align::Right => pos.x - width,
        }
        .round();

        for c in text.chars() {
            let Some(glyph) = font.glyph(c) else {
                continue;
            };

            if glyph.size.x > 0.0 {
                self.quad(
                    Rect::new(
                        x + glyph.offset.x * scale,
                        (pos.y + font.cap * scale * 0.5).round() + glyph.offset.y * scale,
                        glyph.size.x * scale,
                        glyph.size.y * scale,
                    ),
                    glyph.uv,
                    color,
                    font.texture,
                );
            }
            x += glyph.advance * scale;
        }
    }

    pub fn paragraph(&mut self, text: &str, rect: Rect, style: TextStyle) {
        let line_height = self.font(style.font).em(style.size) * 1.4;

        for (i, line) in self
            .font(style.font)
            .wrap(text, style.size, rect.w)
            .iter()
            .enumerate()
        {
            self.text(
                line,
                Vec2::new(
                    match style.align {
                        Align::Left => rect.x,
                        Align::Center => rect.center().x,
                        Align::Right => rect.right(),
                    },
                    rect.y + line_height * (i as f32 + 0.5),
                ),
                style,
            );
        }
    }
}
