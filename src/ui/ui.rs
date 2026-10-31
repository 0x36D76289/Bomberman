use crate::{
    audio::{AudioManager, SoundEffect},
    game::resources::Resources,
    graphics::{StateRenderInfo, UiVertex},
    ui::{draw::DrawList, input::UiInput, layout::Rect},
};
use glam::{IVec2, Vec2};
use std::collections::HashSet;

pub type Id = u64;

pub fn id(name: &str) -> Id {
    name.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

pub fn idx(name: &str, index: usize) -> Id {
    id(name) ^ (index as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Button,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
struct WidgetInfo {
    id: Id,
    rect: Rect,
    kind: Kind,
}

#[derive(Debug, Clone, Default)]
pub struct UiState {
    pub time: f32,
    pub focus: Option<Id>,
    widgets: Vec<WidgetInfo>,
    pressed: Option<Id>,
    flags: HashSet<Id>,
    pub verts: Vec<UiVertex>,
    pub render_info: StateRenderInfo,
}

impl UiState {
    pub fn restart(&mut self) {
        self.time = 0.0;
    }

    pub fn memory_flag(&self, id: Id) -> bool {
        self.flags.contains(&id)
    }

    pub fn set_memory_flag(&mut self, id: Id, value: bool) {
        if value {
            self.flags.insert(id);
        } else {
            self.flags.remove(&id);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Response {
    pub focused: bool,
    pub hovered: bool,
    pub clicked: bool,
    pub step: i32,
    pub rect: Rect,
}

fn navigate(widgets: &[WidgetInfo], current: Option<Id>, dir: IVec2) -> Option<Id> {
    let Some(from) = current.and_then(|c| widgets.iter().find(|w| w.id == c)) else {
        return widgets.first().map(|w| w.id);
    };
    let dir = dir.as_vec2().normalize_or_zero();
    let center = from.rect.center();
    let others = widgets.iter().filter(|w| w.id != from.id);

    let score = |w: &&WidgetInfo| {
        let d = w.rect.center() - center;
        let (a, b) = if dir.x != 0.0 {
            (
                (w.rect.y, w.rect.bottom()),
                (from.rect.y, from.rect.bottom()),
            )
        } else {
            ((w.rect.x, w.rect.right()), (from.rect.x, from.rect.right()))
        };
        d.dot(dir)
            + (a.0 - b.1).max(b.0 - a.1).max(0.0) * 2.5
            + d.dot(Vec2::new(-dir.y, dir.x)).abs() * 0.01
    };
    others
        .clone()
        .filter(|w| (w.rect.center() - center).dot(dir) > 1.0)
        .min_by(|a, b| score(a).total_cmp(&score(b)))
        .or_else(|| {
            if dir.y != 0.0 {
                others
                    .clone()
                    .filter(|w| (w.rect.center() - center).dot(dir) < -1.0)
                    .min_by(|a, b| score(a).total_cmp(&score(b)))
            } else {
                None
            }
        })
        .map(|w| w.id)
}

pub struct Ui<'a> {
    pub state: &'a mut UiState,
    pub dl: DrawList,
    pub input: UiInput,
    widgets: Vec<WidgetInfo>,
    sounds: Vec<SoundEffect>,
    step: i32,
    default_focus: Option<Id>,
}

impl<'a> Ui<'a> {
    pub fn begin(
        state: &'a mut UiState,
        input: &UiInput,
        resources: &'static Resources,
        screen: Vec2,
        dt: f32,
    ) -> Self {
        state.time += dt;
        let mut sounds = Vec::new();

        let mut nav = input.nav;
        let mut step = 0;
        if state
            .widgets
            .iter()
            .find(|w| Some(w.id) == state.focus)
            .map(|w| w.kind)
            == Some(Kind::Horizontal)
            && nav.x != 0
        {
            step = nav.x;
            nav.x = 0;
        }
        if nav != IVec2::ZERO
            && let Some(next) = navigate(&state.widgets, state.focus, nav)
            && Some(next) != state.focus
        {
            state.focus = Some(next);
            sounds.push(SoundEffect::MenuMove);
        }

        if let Some(mouse) = input.mouse {
            let hovered = state.widgets.iter().rev().find(|w| w.rect.contains(mouse));
            if (input.mouse_moved || input.mouse_pressed)
                && let Some(w) = hovered
                && state.focus != Some(w.id)
            {
                state.focus = Some(w.id);
                sounds.push(SoundEffect::MenuMove);
            }
            if input.mouse_pressed {
                state.pressed = hovered.map(|w| w.id);
            }
        }

        Self {
            state,
            dl: DrawList::new(screen, resources),
            input: *input,
            widgets: Vec::new(),
            sounds,
            step,
            default_focus: None,
        }
    }

    pub fn screen(&self) -> Vec2 {
        self.dl.screen
    }

    pub fn time(&self) -> f32 {
        self.state.time
    }

    pub fn default_focus(&mut self, id: Id) {
        self.default_focus = Some(id);
    }

    pub fn sound(&mut self, effect: SoundEffect) {
        if !self.sounds.contains(&effect) {
            self.sounds.push(effect);
        }
    }

    pub fn back_pressed(&mut self) -> bool {
        if self.input.back {
            self.sound(SoundEffect::MenuBack);
        }
        self.input.back
    }

    pub fn interact(&mut self, id: Id, rect: Rect, kind: Kind) -> Response {
        self.widgets.push(WidgetInfo { id, rect, kind });
        let focused = self.state.focus == Some(id);
        let hovered = self.input.mouse.is_some_and(|m| rect.contains(m));

        let mut clicked = focused && self.input.confirm;
        if self.input.mouse_released && hovered && self.state.pressed == Some(id) {
            clicked = true;
        }
        if clicked {
            self.sound(SoundEffect::MenuConfirm);
        }
        Response {
            focused,
            hovered,
            clicked,
            step: if focused { self.step } else { 0 },
            rect,
        }
    }

    pub fn dragging(&self, id: Id) -> bool {
        self.input.mouse_down && self.state.pressed == Some(id)
    }

    pub fn end(mut self, audio: &AudioManager) {
        if self.input.mouse_released {
            self.state.pressed = None;
        }
        if !self
            .state
            .focus
            .is_some_and(|f| self.widgets.iter().any(|w| w.id == f))
        {
            self.state.focus = self
                .default_focus
                .filter(|d| self.widgets.iter().any(|w| w.id == *d))
                .or_else(|| self.widgets.first().map(|w| w.id));
        }
        self.state.widgets = std::mem::take(&mut self.widgets);
        self.state.verts = std::mem::take(&mut self.dl.verts);
        for sound in self.sounds {
            audio.play_sound_effect(sound);
        }
    }
}
