use crate::game::{bot::BotLevel, map::map_settings::MapSettings, theme::Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyPreset {
    Corners,
    Arena,
    Teams,
    Custom,
}

impl LobbyPreset {
    pub fn name(self) -> &'static str {
        match self {
            LobbyPreset::Corners => "Coins",
            LobbyPreset::Arena => "Arène",
            LobbyPreset::Teams => "2v2v2v2",
            LobbyPreset::Custom => "Libre",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            LobbyPreset::Corners => "Le classique : 4 joueurs, un dans chaque coin",
            LobbyPreset::Arena => "Un grand champ de bataille jusqu'à 10 joueurs",
            LobbyPreset::Teams => "4 équipes de 2 : clique sur une case pour la changer",
            LobbyPreset::Custom => "Toutes tailles, départs aléatoires",
        }
    }

    fn base(self) -> MapSettings {
        match self {
            LobbyPreset::Corners => MapSettings::corners(),
            LobbyPreset::Arena => MapSettings::arena(),
            LobbyPreset::Teams => MapSettings::teams(),
            LobbyPreset::Custom => MapSettings::random(),
        }
    }

    pub fn max_players(self) -> u8 {
        match self {
            LobbyPreset::Corners => 4,
            LobbyPreset::Arena => 10,
            LobbyPreset::Teams => 8,
            LobbyPreset::Custom => 12,
        }
    }

    fn size_limits(self) -> (u8, u8, u8) {
        match self {
            LobbyPreset::Corners => (7, 31, 2),
            LobbyPreset::Arena => (13, 41, 4),
            LobbyPreset::Teams => (11, 31, 2),
            LobbyPreset::Custom => (9, 31, 2),
        }
    }
}

pub const TEAM_COUNT: usize = 4;
pub const TEAM_SIZE: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Empty,
    Bot,
    Human(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobbySettings {
    pub preset: LobbyPreset,
    pub width: u8,
    pub height: u8,
    pub cheesiness: u8,
    pub humans: u8,
    pub bots: u8,
    pub bot_level: BotLevel,
    pub theme: Theme,
    pub slots: [Slot; TEAM_COUNT * TEAM_SIZE],
}

impl LobbySettings {
    pub fn new(preset: LobbyPreset, humans: u8) -> Self {
        let base = preset.base();
        let humans = humans.clamp(1, preset.max_players());
        let mut lobby = Self {
            preset,
            width: base.width,
            height: base.height,
            cheesiness: base.cheesiness,
            humans,
            bots: (4u8.min(preset.max_players())).saturating_sub(humans),
            bot_level: BotLevel::default(),
            theme: Theme::Arena,
            slots: [Slot::Bot; TEAM_COUNT * TEAM_SIZE],
        };
        if lobby.is_teams() {
            lobby.sync_slots();
        }
        lobby
    }

    pub fn is_teams(&self) -> bool {
        self.preset == LobbyPreset::Teams
    }

    pub fn set_humans(&mut self, humans: u8) {
        self.humans = humans;
        if self.is_teams() {
            self.sync_slots();
        } else {
            self.step_bots(0);
        }
    }

    fn sync_slots(&mut self) {
        for slot in self.slots.iter_mut() {
            if matches!(slot, Slot::Human(k) if *k >= self.humans) {
                *slot = Slot::Empty;
            }
        }
        for k in 0..self.humans {
            if self.slots.contains(&Slot::Human(k)) {
                continue;
            }
            if let Some(s) = (0..TEAM_SIZE)
                .flat_map(|member| (0..TEAM_COUNT).map(move |team| team * TEAM_SIZE + member))
                .filter(|s| !matches!(self.slots[*s], Slot::Human(_)))
                .min_by_key(|s| self.slots[*s] != Slot::Empty)
            {
                self.slots[s] = Slot::Human(k);
            }
        }
        self.bots = self.slots.iter().filter(|s| **s == Slot::Bot).count() as u8;
    }

    pub fn cycle_slot(&mut self, index: usize) {
        match self.slots[index] {
            Slot::Empty => self.slots[index] = Slot::Bot,
            Slot::Bot => self.slots[index] = Slot::Empty,
            Slot::Human(_) => {
                if let Some(target) = (1..TEAM_COUNT)
                    .map(|offset| (index / TEAM_SIZE + offset) % TEAM_COUNT)
                    .flat_map(|other| (0..TEAM_SIZE).map(move |m| other * TEAM_SIZE + m))
                    .find(|s| !matches!(self.slots[*s], Slot::Human(_)))
                {
                    self.slots.swap(index, target);
                }
            }
        }
        self.sync_slots();
    }

    pub fn player_teams(&self) -> Option<Vec<u32>> {
        if !self.is_teams() {
            return None;
        }
        let team_of = |wanted: Slot| {
            self.slots
                .iter()
                .enumerate()
                .filter(move |(_, slot)| **slot == wanted)
                .map(|(s, _)| (s / TEAM_SIZE) as u32)
        };
        Some(
            (0..self.humans)
                .flat_map(|k| team_of(Slot::Human(k)))
                .chain(team_of(Slot::Bot))
                .collect(),
        )
    }

    pub fn set_preset(&mut self, preset: LobbyPreset) {
        *self = Self {
            bot_level: self.bot_level,
            theme: self.theme,
            ..Self::new(preset, self.humans)
        };
    }

    pub fn step_width(&mut self, steps: i32) {
        let (min, max, step) = self.preset.size_limits();
        self.width = (self.width as i32 + steps * step as i32).clamp(min as i32, max as i32) as u8;
    }

    pub fn step_height(&mut self, steps: i32) {
        let (min, max, step) = self.preset.size_limits();
        self.height =
            (self.height as i32 + steps * step as i32).clamp(min as i32, max as i32) as u8;
    }

    pub fn step_bots(&mut self, steps: i32) {
        if self.is_teams() {
            for _ in 0..steps.unsigned_abs() {
                let (from, to) = if steps > 0 {
                    (Slot::Empty, Slot::Bot)
                } else {
                    (Slot::Bot, Slot::Empty)
                };
                if let Some(s) = if steps > 0 {
                    self.slots.iter().position(|s| *s == from)
                } else {
                    self.slots.iter().rposition(|s| *s == from)
                } {
                    self.slots[s] = to;
                }
            }
            self.sync_slots();
            return;
        }
        self.bots = (self.bots as i32 + steps).clamp(
            0,
            self.preset.max_players().saturating_sub(self.humans) as i32,
        ) as u8;
    }

    pub fn step_bot_level(&mut self, steps: i32) {
        self.bot_level = self.bot_level.stepped(steps);
    }

    pub fn step_theme(&mut self, steps: i32) {
        self.theme = Theme::ALL[(Theme::ALL
            .iter()
            .position(|t| *t == self.theme)
            .unwrap_or(0) as i32
            + steps)
            .rem_euclid(Theme::ALL.len() as i32) as usize];
    }

    pub fn players(&self) -> u8 {
        self.humans + self.bots
    }

    pub fn map_settings(&self) -> MapSettings {
        MapSettings {
            width: self.width,
            height: self.height,
            cheesiness: self.cheesiness,
            spawns: self.players(),
            theme: self.theme,
            ..self.preset.base()
        }
    }
}
