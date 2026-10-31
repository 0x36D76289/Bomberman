use crate::{
    audio::SoundEffect,
    graphics::text::FontId,
    ui::{
        draw::{Align, TextStyle},
        input::Device,
        layout::Rect,
        style,
        textures::{Icon, Prompt, Tint, UiTex},
        ui::{Id, Kind, Response, Ui},
    },
};
use glam::{Vec2, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Normal,
    Danger,
    Go,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Confirm,
    Back,
    Navigate,
    Change,
    Pause,
}

impl Action {
    fn prompt(self, device: Device) -> Prompt {
        match (self, device) {
            (Action::Confirm, Device::Gamepad) => Prompt::PadA,
            (Action::Confirm, Device::Mouse) => Prompt::MouseLeft,
            (Action::Confirm, Device::Keyboard) => Prompt::KeySpace,
            (Action::Back | Action::Pause, Device::Gamepad) => Prompt::PadB,
            (Action::Back | Action::Pause, _) => Prompt::KeyEscape,
            (Action::Navigate | Action::Change, Device::Gamepad) => Prompt::PadDpad,
            (Action::Change, Device::Mouse) => Prompt::MouseScroll,
            (Action::Navigate | Action::Change, _) => Prompt::KeyArrows,
        }
    }
}

fn button_text(size: f32, color: Vec4) -> TextStyle {
    TextStyle::new(FontId::Title, size, color).shadow(
        if color == style::INK {
            Vec4::ZERO
        } else {
            style::INK
        },
        Vec2::splat(3.0),
    )
}

impl Ui<'_> {
    fn cursor(&mut self, rect: Rect, color: Vec4) {
        self.dl
            .outline(rect.inset(-style::PIXEL * 2.0), style::PIXEL, color);
    }

    pub fn button(
        &mut self,
        id: Id,
        rect: Rect,
        text: &str,
        icon: Option<Icon>,
        look: ButtonStyle,
    ) -> Response {
        let mut r = self.interact(id, rect, Kind::Button);
        if look == ButtonStyle::Disabled && r.clicked {
            r.clicked = false;
            self.sound(SoundEffect::NoBomb);
        }
        if r.focused {
            self.cursor(rect, style::WHITE);
        }
        self.dl.frame(
            UiTex::Button(match look {
                ButtonStyle::Disabled => Tint::Grey,
                _ if r.focused => Tint::Yellow,
                ButtonStyle::Normal => Tint::Blue,
                ButtonStyle::Danger => Tint::Red,
                ButtonStyle::Go => Tint::Green,
            }),
            rect,
            Vec4::ONE,
            0.5,
        );

        let mut size = (rect.h * 0.48).min(40.0);
        while size > 8.0
            && self.dl.text_width(text, &button_text(size, style::WHITE))
                > rect.w
                    - if icon.is_some() {
                        rect.h * 1.05 + 16.0
                    } else {
                        32.0
                    }
        {
            size -= 2.0;
        }
        let text_y = rect.center().y;
        let color = if look == ButtonStyle::Disabled {
            style::TEXT_MUTED
        } else if r.focused {
            style::INK
        } else {
            style::WHITE
        };
        let text_style = button_text(size, color);

        match icon {
            Some(icon) => {
                let icon_rect = Rect::centered(
                    Vec2::new(rect.x + rect.h * 0.55, text_y),
                    Vec2::splat(rect.h * 0.5),
                );
                self.dl.image(
                    UiTex::Icon(icon),
                    icon_rect.translate(Vec2::splat(3.0)),
                    text_style.shadow.map_or(Vec4::ZERO, |s| s.0),
                );
                self.dl.image(UiTex::Icon(icon), icon_rect, color);
                self.dl.text(
                    text,
                    Vec2::new(rect.x + rect.h * 1.05, text_y),
                    text_style.align(Align::Left),
                );
            }
            None => self
                .dl
                .text(text, Vec2::new(rect.center().x, text_y), text_style),
        }
        r
    }

    pub fn back_button(&mut self, rect: Rect) -> bool {
        self.button(
            crate::ui::ui::id("back"),
            rect,
            "Retour",
            Some(Icon::Return),
            ButtonStyle::Normal,
        )
        .clicked
            || self.back_pressed()
    }

    fn row_background(&mut self, r: &Response) {
        self.dl.frame(UiTex::Panel, r.rect, Vec4::ONE, 0.5);
        if r.focused {
            self.dl.outline(r.rect, style::PIXEL, style::YELLOW);
            self.dl.image(
                UiTex::ArrowRight,
                Rect::new(r.rect.x + 8.0, r.rect.center().y - 10.0, 10.0, 20.0),
                style::YELLOW,
            );
        }
    }

    fn row_label(&mut self, rect: Rect, label: &str) {
        self.dl.text(
            label,
            Vec2::new(rect.x + 28.0, rect.center().y),
            TextStyle::new(FontId::Title, style::TEXT, style::TEXT_DARK).align(Align::Left),
        );
    }

    pub fn toggle(&mut self, id: Id, rect: Rect, label: &str, value: &mut bool) -> bool {
        let r = self.interact(id, rect, Kind::Horizontal);

        let changed = r.clicked || r.step != 0;
        if changed {
            *value = !*value;
            self.sound(SoundEffect::MenuSwitch);
        }

        self.row_background(&r);
        self.row_label(rect, label);

        let size = rect.h * 0.62;
        let check = Rect::centered(
            Vec2::new(rect.right() - 28.0 - size / 2.0, rect.center().y),
            Vec2::splat(size),
        );

        self.dl.image(
            if *value {
                UiTex::CheckOn
            } else {
                UiTex::CheckOff
            },
            check,
            Vec4::ONE,
        );
        self.dl.text(
            if *value { "Oui" } else { "Non" },
            Vec2::new(check.x - 18.0, rect.center().y),
            TextStyle::new(FontId::Title, style::TEXT, style::TEXT_MUTED).align(Align::Right),
        );
        changed
    }

    pub fn slider(&mut self, id: Id, rect: Rect, label: &str, value: &mut f32) -> bool {
        let r = self.interact(id, rect, Kind::Horizontal);
        let track = Rect::new(
            rect.x + rect.w * 0.45,
            rect.center().y - 14.0,
            rect.w * 0.55 - 120.0,
            28.0,
        );

        let old = *value;
        *value = (*value + r.step as f32 * 0.05).clamp(0.0, 1.0);

        if r.hovered && self.input.wheel != 0.0 {
            *value = (*value + self.input.wheel.signum() * 0.05).clamp(0.0, 1.0);
        }
        if self.dragging(id)
            && let Some(mouse) = self.input.mouse
        {
            *value = ((mouse.x - track.x) / track.w).clamp(0.0, 1.0);
        }

        *value = (*value * 20.0).round() / 20.0;

        self.row_background(&r);
        self.row_label(rect, label);

        let bar = Rect::new(track.x, track.center().y - 12.0, track.w, 24.0);

        self.dl.frame(UiTex::PanelGrey, bar, Vec4::ONE, 0.5);

        let inner = bar.inset(style::PIXEL);
        let segments = 20;
        let segment = inner.w / segments as f32;
        for i in 0..(*value * segments as f32).round() as usize {
            self.dl.rect(
                Rect::new(
                    inner.x + i as f32 * segment,
                    inner.y,
                    segment - 2.0,
                    inner.h,
                ),
                style::YELLOW,
            );
        }

        self.dl.frame(
            UiTex::Square(if r.focused { Tint::Yellow } else { Tint::Grey }),
            Rect::centered(
                Vec2::new(track.x + track.w * *value, track.center().y),
                Vec2::new(24.0, 40.0),
            ),
            Vec4::ONE,
            0.4,
        );
        self.dl.text(
            &format!("{}%", (*value * 100.0).round() as u32),
            Vec2::new(rect.right() - 28.0, rect.center().y),
            TextStyle::new(FontId::Title, style::TEXT, style::TEXT_DARK).align(Align::Right),
        );

        (*value - old).abs() > f32::EPSILON
    }

    pub fn selector(&mut self, id: Id, rect: Rect, label: &str, value: &str) -> i32 {
        let r = self.interact(id, rect, Kind::Horizontal);
        let value_center = Vec2::new(rect.x + rect.w * 0.72, rect.center().y);
        let arrow_size = Vec2::splat(rect.h * 0.55);
        let left = Rect::centered(value_center - Vec2::new(rect.w * 0.2, 0.0), arrow_size);

        let mut step = r.step;
        if r.clicked {
            step = 1;
            if let Some(mouse) = self.input.mouse
                && left.inset(-10.0).contains(mouse)
            {
                step = -1;
            }
        }
        if r.hovered && self.input.wheel != 0.0 {
            step = self.input.wheel.signum() as i32;
        }
        if step != 0 {
            self.sound(SoundEffect::MenuSwitch);
        }

        self.row_background(&r);
        self.row_label(rect, label);

        let arrow = if r.focused {
            style::YELLOW
        } else {
            style::WHITE
        };
        let arrow_box = Vec2::new(arrow_size.x * 0.5, arrow_size.y * 0.7);
        self.dl.image(
            UiTex::ArrowLeft,
            Rect::centered(left.center(), arrow_box),
            arrow,
        );
        self.dl.image(
            UiTex::ArrowRight,
            Rect::centered(
                Rect::centered(value_center + Vec2::new(rect.w * 0.2, 0.0), arrow_size).center(),
                arrow_box,
            ),
            arrow,
        );
        self.dl.text(
            value,
            value_center,
            TextStyle::new(FontId::Title, style::TEXT, style::TEXT_DARK),
        );

        step
    }

    pub fn card(
        &mut self,
        id: Id,
        rect: Rect,
        selected: bool,
        content: impl FnOnce(&mut Self, &Response),
    ) -> Response {
        let r = self.interact(id, rect, Kind::Button);

        self.dl.frame(UiTex::Panel, rect, Vec4::ONE, 0.5);
        if selected {
            self.dl.outline(rect, style::PIXEL, style::GREEN);
        }
        if r.focused {
            self.cursor(rect, style::YELLOW);
        }
        content(self, &r);
        r
    }

    pub fn panel(&mut self, rect: Rect) {
        self.dl.frame(UiTex::Panel, rect, Vec4::ONE, 0.75);
    }

    pub fn fill(&mut self, color: Vec4) {
        let screen = self.screen();
        self.dl.rect(Rect::new(0.0, 0.0, screen.x, screen.y), color);
    }

    pub fn backdrop(&mut self) {
        let screen = self.screen();
        self.dl.gradient(
            Rect::new(0.0, 0.0, screen.x, screen.y),
            style::BACKDROP_TOP,
            style::BACKDROP_BOTTOM,
        );
        let hash = |n: u32| ((n.wrapping_mul(2_654_435_761) >> 8) % 10_000) as f32 / 10_000.0;
        for i in 0..90u32 {
            let size = style::PIXEL * (1.0 + (hash(i * 7 + 5) * 2.0).floor());
            self.dl.rect(
                Rect::new(
                    (hash(i * 3) * screen.x / style::PIXEL).floor() * style::PIXEL,
                    (hash(i * 3 + 1) * screen.y / style::PIXEL).floor() * style::PIXEL,
                    size,
                    size,
                ),
                style::fade(style::WHITE, (0.35 + hash(i * 11) * 0.55) * 0.6),
            );
        }
        self.scanlines();
    }

    pub fn scanlines(&mut self) {
        let screen = self.screen();
        let mut y = 0.0;
        while y < screen.y {
            self.dl.rect(
                Rect::new(0.0, y, screen.x, 2.0),
                style::fade(style::INK, 0.18),
            );
            y += 4.0;
        }
    }

    pub fn title(&mut self, text: &str, center: Vec2, size: f32) {
        let step = (self.dl.font(FontId::Title).em(size) / 8.0).round();
        self.dl.text(
            text,
            center,
            TextStyle::new(FontId::Title, size, style::YELLOW)
                .outline(style::RED, step)
                .shadow(style::INK, Vec2::splat(step * 2.0)),
        );
    }

    pub fn header(&mut self, text: &str) {
        self.dl.text(
            text,
            Vec2::new(self.screen().x / 2.0, 90.0),
            TextStyle::new(FontId::Title, style::HEADER, style::WHITE).outline(
                style::INK,
                (self.dl.font(FontId::Title).em(style::HEADER) / 8.0).round(),
            ),
        );
    }

    pub fn prompts(&mut self, actions: &[(Action, &str)]) {
        let screen = self.screen();
        let text_style = TextStyle::new(FontId::Title, 24.0, style::WHITE)
            .align(Align::Left)
            .outline(style::fade(style::INK, 0.8), 3.0);
        let icon = 52.0;
        let widths: Vec<f32> = actions
            .iter()
            .map(|(_, text)| icon + 8.0 + self.dl.text_width(text, &text_style) + 28.0)
            .collect();
        let mut x = screen.x - style::MARGIN - widths.iter().sum::<f32>();
        let y = screen.y - 52.0;
        for ((action, text), width) in actions.iter().zip(widths) {
            self.dl.image_fit(
                UiTex::Prompt(action.prompt(self.input.device)),
                Rect::centered(Vec2::new(x + icon / 2.0, y), Vec2::splat(icon)),
                Vec4::ONE,
            );
            self.dl.text(text, Vec2::new(x + icon + 8.0, y), text_style);
            x += width;
        }
    }

    pub fn dialog(&mut self, name: &str, title: &str, message: &str, accept: &str) -> Option<bool> {
        self.fill(style::OVERLAY);
        let rect = Rect::centered(self.screen() / 2.0, Vec2::new(860.0, 400.0));
        self.dl.frame(UiTex::Panel, rect, Vec4::ONE, 0.6);
        self.dl.text(
            title,
            Vec2::new(rect.center().x, rect.y + 70.0),
            TextStyle::new(FontId::Title, 52.0, style::YELLOW).outline(style::INK, 4.0),
        );
        self.dl.paragraph(
            message,
            Rect::new(rect.x + 60.0, rect.y + 120.0, rect.w - 120.0, 120.0),
            TextStyle::new(FontId::Body, style::TEXT, style::TEXT_DARK),
        );
        let buttons =
            Rect::new(rect.x + 50.0, rect.bottom() - 120.0, rect.w - 100.0, 84.0).columns(2, 30.0);
        let cancel_id = crate::ui::ui::id(&format!("{name}-cancel"));
        self.default_focus(cancel_id);
        if self
            .button(
                cancel_id,
                buttons[0],
                "Annuler",
                Some(Icon::Return),
                ButtonStyle::Normal,
            )
            .clicked
        {
            return Some(false);
        }
        if self
            .button(
                crate::ui::ui::id(&format!("{name}-accept")),
                buttons[1],
                accept,
                Some(Icon::Checkmark),
                ButtonStyle::Danger,
            )
            .clicked
        {
            return Some(true);
        }
        if self.back_pressed() {
            return Some(false);
        }
        None
    }

    pub fn stars(&mut self, center: Vec2, count: usize, max: usize, size: f32) {
        for i in 0..max {
            self.dl.image_fit(
                if i < count {
                    UiTex::Star
                } else {
                    UiTex::StarEmpty
                },
                Rect::centered(
                    Vec2::new(
                        center.x - (max as f32 - 1.0) * size * 0.55 + i as f32 * size * 1.1,
                        center.y,
                    ),
                    Vec2::splat(size),
                ),
                Vec4::ONE,
            );
        }
    }
}
