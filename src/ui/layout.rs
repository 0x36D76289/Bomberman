use glam::Vec2;

pub const SCREEN_HEIGHT: f32 = 1080.0;

pub fn screen_size(ratio: f32) -> Vec2 {
    Vec2::new(SCREEN_HEIGHT * ratio.max(0.1), SCREEN_HEIGHT)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn centered(center: Vec2, size: Vec2) -> Self {
        Self::new(
            center.x - size.x / 2.0,
            center.y - size.y / 2.0,
            size.x,
            size.y,
        )
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.x <= self.right() && p.y >= self.y && p.y <= self.bottom()
    }

    pub fn inset(&self, amount: f32) -> Self {
        Self::new(
            self.x + amount,
            self.y + amount,
            (self.w - amount * 2.0).max(0.0),
            (self.h - amount * 2.0).max(0.0),
        )
    }

    pub fn translate(&self, offset: Vec2) -> Self {
        Self::new(self.x + offset.x, self.y + offset.y, self.w, self.h)
    }

    pub fn rows(&self, count: usize, gap: f32) -> Vec<Rect> {
        let h = (self.h - gap * (count.max(1) - 1) as f32) / count.max(1) as f32;
        (0..count)
            .map(|i| Rect::new(self.x, self.y + i as f32 * (h + gap), self.w, h))
            .collect()
    }

    pub fn columns(&self, count: usize, gap: f32) -> Vec<Rect> {
        let w = (self.w - gap * (count.max(1) - 1) as f32) / count.max(1) as f32;
        (0..count)
            .map(|i| Rect::new(self.x + i as f32 * (w + gap), self.y, w, self.h))
            .collect()
    }

    pub fn take_top(&self, h: f32) -> (Rect, Rect) {
        (
            Rect::new(self.x, self.y, self.w, h),
            Rect::new(self.x, self.y + h, self.w, (self.h - h).max(0.0)),
        )
    }
}
