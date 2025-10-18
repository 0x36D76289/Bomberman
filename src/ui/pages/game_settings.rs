use glam::{Vec2, Vec4, usize};

use crate::{
    app_state::AppState,
    game::{game_state::GameState, map::map_settings::MapSettings, resources::Resources},
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
        ui_state::UIPage,
    },
};

use super::super::consts::*;
use super::super::utils::*;

const PRESET_BUTTON_SIZE: Vec2 = Vec2::new(0.4, 0.3);
const PRESET_SPACING: f32 = 0.05;

const PRESET_VERTICAL_HEIGHT: f32 = -0.65;

const SETTINGS_SIZE: Vec2 = Vec2::new(1.0, 0.2);
const SETTING_GRID_COUNT: u8 = 8;
const SETTING_GRID_START_INDEX: u8 = 2;

fn create_outlined_button(
    pos: Vec2,
    size: Vec2,
    neighbors: ButtonNeighbors,
    buttons: &mut Vec<Button>,
    text: &'static str,
) {
    buttons.push(Button {
        canvas: Canvas {
            center: pos,
            width: size.x,
            height: size.y,
            color: BUTTON_COLOR,
            texture: None,
            text: Some(text.to_string()),
            text_color: Some(TEXT_COLOR),
            text_size: Some(TEXT_SIZE),
        },
        outline_color: Some(OUTLINE_COLOR),
        neighbors: neighbors,
        selected_color: SELECTED_BUTTON_COLOR,
        selected_text_color: Some(SELECTED_TEXT_COLOR),
        selected_texture: None,
    });
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GameSettingPreset {
    Corners,
    Arena,
    Teams,
    Custom,
}

#[derive(Debug, Copy, Clone)]
pub struct UIGameSettings {
    preset: GameSettingPreset,
    /// map width
    width: u8,
    /// map height
    height: u8,
    /// percentage of removable blocks that are missing at round start
    cheesiness: u8,
    /// amount of human players
    player_count: u8,
    /// amount of ai players
    bot_count: u8,
    /// alpha channel of error message multiplied by ERROR_VISIBILITY_TIME
    opacity: f32,
}

impl UIGameSettings {
    fn corners(player_count: u8) -> Self {
        let mut bot_count = 4;

        if player_count > bot_count {
            bot_count = 0;
        } else {
            bot_count -= player_count;
        }

        let map_settings = MapSettings::corners();

        Self {
            preset: GameSettingPreset::Corners,
            width: map_settings.width,
            height: map_settings.height,
            cheesiness: map_settings.cheesiness,
            player_count,
            bot_count,
            opacity: 0.0,
        }
    }
    fn arena(player_count: u8) -> Self {
        let mut bot_count = 10;

        if player_count > bot_count {
            bot_count = 0;
        } else {
            bot_count -= player_count;
        }

        let map_settings = MapSettings::arena();

        Self {
            preset: GameSettingPreset::Arena,
            width: map_settings.width,
            height: map_settings.height,
            cheesiness: map_settings.cheesiness,
            player_count,
            bot_count,
            opacity: 0.0,
        }
    }
    fn teams(player_count: u8) -> Self {
        let map_settings = MapSettings::teams();

        Self {
            preset: GameSettingPreset::Teams,
            width: map_settings.width,
            height: map_settings.height,
            cheesiness: map_settings.cheesiness,
            player_count,
            bot_count: 0,
            opacity: 0.0,
        }
    }
}

enum GameSettingButtons {
    PresetCorners,
    PresetArena,
    PresetTeams,
    PresetCustom,
    SettingWidth,
    SettingHeight,
    SettingCheese,
    SettingBotCount,
    Start,
    LabelWidth,
    LabelHeight,
    LabelCheese,
    LabelBotCount,
}

impl UiState {
    pub fn game_settings(player_count: u8) -> Self {
        let mut canvases = Vec::<Canvas>::new();
        let mut buttons = Vec::<Button>::new();

        canvases.push(Canvas {
            center: Vec2::ZERO,
            width: 2.0,
            height: 2.0,
            color: BACKGROUND_COLOR,
            texture: None,
            text: None,
            text_color: None,
            text_size: None,
        });

        // Corners
        create_outlined_button(
            Vec2 {
                x: -PRESET_SPACING * 1.5 - PRESET_BUTTON_SIZE.x * 1.5,
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::PresetCorners as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::PresetCorners as usize,
                right: GameSettingButtons::PresetArena as usize,
            },
            &mut buttons,
            "Corners",
        );
        buttons[0].toggle();

        // PresetArena
        create_outlined_button(
            Vec2 {
                x: -PRESET_SPACING * 0.5 - PRESET_BUTTON_SIZE.x * 0.5,
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::PresetArena as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::PresetCorners as usize,
                right: GameSettingButtons::PresetTeams as usize,
            },
            &mut buttons,
            "Arena",
        );

        // PresetTeams
        create_outlined_button(
            Vec2 {
                x: PRESET_SPACING * 0.5 + PRESET_BUTTON_SIZE.x * 0.5,
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::PresetTeams as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::PresetArena as usize,
                right: GameSettingButtons::PresetCustom as usize,
            },
            &mut buttons,
            "2v2v2v2",
        );

        // Custom
        create_outlined_button(
            Vec2 {
                x: PRESET_SPACING * 1.5 + PRESET_BUTTON_SIZE.x * 1.5,
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::PresetCustom as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::PresetTeams as usize,
                right: GameSettingButtons::PresetCustom as usize,
            },
            &mut buttons,
            "Custom",
        );

        // width
        create_outlined_button(
            Vec2 {
                x: 0.0,
                y: spread(SETTING_GRID_COUNT, SETTING_GRID_START_INDEX + 0),
            },
            SETTINGS_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::PresetCustom as usize,
                down: GameSettingButtons::SettingHeight as usize,
                left: GameSettingButtons::SettingWidth as usize,
                right: GameSettingButtons::SettingWidth as usize,
            },
            &mut buttons,
            "Board Width",
        );

        // height
        create_outlined_button(
            Vec2 {
                x: 0.0,
                y: spread(SETTING_GRID_COUNT, SETTING_GRID_START_INDEX + 1),
            },
            SETTINGS_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::SettingWidth as usize,
                down: GameSettingButtons::SettingCheese as usize,
                left: GameSettingButtons::SettingHeight as usize,
                right: GameSettingButtons::SettingHeight as usize,
            },
            &mut buttons,
            "Board Height",
        );

        // cheese
        create_outlined_button(
            Vec2 {
                x: 0.0,
                y: spread(SETTING_GRID_COUNT, SETTING_GRID_START_INDEX + 2),
            },
            SETTINGS_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::SettingHeight as usize,
                down: GameSettingButtons::SettingBotCount as usize,
                left: GameSettingButtons::SettingCheese as usize,
                right: GameSettingButtons::SettingCheese as usize,
            },
            &mut buttons,
            "Cheesiness",
        );

        // bot count
        create_outlined_button(
            Vec2 {
                x: 0.0,
                y: spread(SETTING_GRID_COUNT, SETTING_GRID_START_INDEX + 3),
            },
            SETTINGS_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::SettingCheese as usize,
                down: GameSettingButtons::Start as usize,
                left: GameSettingButtons::SettingBotCount as usize,
                right: GameSettingButtons::SettingBotCount as usize,
            },
            &mut buttons,
            "Bot Count",
        );

        // Start
        create_outlined_button(
            Vec2 {
                x: 0.0,
                y: -PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::SettingBotCount as usize,
                down: GameSettingButtons::Start as usize,
                left: GameSettingButtons::Start as usize,
                right: GameSettingButtons::Start as usize,
            },
            &mut buttons,
            "Start",
        );

        let settings = UIGameSettings::corners(player_count);
        // Width Label
        buttons.push(Button {
            canvas: Canvas {
                center: Vec2 {
                    x: 0.4,
                    y: spread(SETTING_GRID_COUNT, SETTING_GRID_START_INDEX + 0),
                },
                width: 0.0,
                height: 0.0,
                color: Vec4::ZERO,
                texture: None,
                text: Some(settings.width.to_string()),
                text_color: Some(OUTLINE_COLOR),
                text_size: Some(TEXT_SIZE),
            },
            ..Default::default()
        });
        // Height Label
        buttons.push(Button {
            canvas: Canvas {
                center: Vec2 {
                    x: 0.4,
                    y: spread(SETTING_GRID_COUNT, SETTING_GRID_START_INDEX + 1),
                },
                width: 0.0,
                height: 0.0,
                color: Vec4::ZERO,
                texture: None,
                text: Some(settings.height.to_string()),
                text_color: Some(OUTLINE_COLOR),
                text_size: Some(TEXT_SIZE),
            },
            ..Default::default()
        });
        // Cheesiness Label
        buttons.push(Button {
            canvas: Canvas {
                center: Vec2 {
                    x: 0.4,
                    y: spread(SETTING_GRID_COUNT, SETTING_GRID_START_INDEX + 2),
                },
                width: 0.0,
                height: 0.0,
                color: Vec4::ZERO,
                texture: None,
                text: Some(settings.cheesiness.to_string()),
                text_color: Some(OUTLINE_COLOR),
                text_size: Some(TEXT_SIZE),
            },
            ..Default::default()
        });
        // Bot Count Label
        buttons.push(Button {
            canvas: Canvas {
                center: Vec2 {
                    x: 0.4,
                    y: spread(SETTING_GRID_COUNT, SETTING_GRID_START_INDEX + 3),
                },
                width: 0.0,
                height: 0.0,
                color: Vec4::ZERO,
                texture: None,
                text: Some(settings.bot_count.to_string()),
                text_color: Some(OUTLINE_COLOR),
                text_size: Some(TEXT_SIZE),
            },
            ..Default::default()
        });

        canvases.push(Canvas {
            center: Vec2 { x: 0.0, y: 0.9 },
            width: 0.0,
            height: 0.0,
            color: Vec4::ZERO,
            texture: None,
            text: Some("Sample text but long error msg".to_string()),
            text_color: Some(ERROR_MESSAGE_COLOR),
            text_size: Some(TEXT_SIZE),
        });

        Self {
            canvases,
            buttons,
            is_transparent: false,
            selected: 0,
            page: UIPage::GameSettings(settings),
        }
    }

    fn update_label_text(&mut self) {
        let UIPage::GameSettings(settings) = self.page else {
            return;
        };
        self.buttons[GameSettingButtons::LabelWidth as usize]
            .canvas
            .text = Some(settings.width.to_string());
        self.buttons[GameSettingButtons::LabelHeight as usize]
            .canvas
            .text = Some(settings.height.to_string());
        self.buttons[GameSettingButtons::LabelCheese as usize]
            .canvas
            .text = Some(settings.cheesiness.to_string());
        self.buttons[GameSettingButtons::LabelBotCount as usize]
            .canvas
            .text = Some(settings.bot_count.to_string());

        for i in
            GameSettingButtons::PresetCorners as usize..=GameSettingButtons::PresetCustom as usize
        {
            self.buttons[i].canvas.text_color = if settings.preset as usize == i {
                Some(HIGHLIGHTED_TEXT_COLOR)
            } else {
                Some(TEXT_COLOR)
            }
        }
    }

    fn update_width(preset: GameSettingPreset, value: &mut u8, modif: i16) -> Option<String> {
        if modif.is_negative() {
            if *value == 5 {
                return Some("Width cannot be below 5".to_string());
            }
            if preset == GameSettingPreset::Arena && *value == 17 {
                return Some("Width cannot be below 17 in Arena mode".to_string());
            }
            if preset == GameSettingPreset::Teams && *value == 11 {
                return Some("Width cannot be below 11 in Teams mode".to_string());
            }
        } else if modif.is_positive() {
            match preset {
                GameSettingPreset::Arena => {
                    if *value == 97 {
                        return Some("Width cannot be over 97 in Arena mode".to_string());
                    }
                }
                _ => {
                    if *value == 99 {
                        return Some("Width cannot be over 99".to_string());
                    }
                }
            }
        }

        let mult = match preset {
            GameSettingPreset::Arena => 4,
            _ => 2,
        };
        *value = (*value as i16 + (modif * mult)) as u8;
        None
    }

    fn update_height(preset: GameSettingPreset, value: &mut u8, modif: i16) -> Option<String> {
        if modif.is_negative() {
            if *value == 5 {
                return Some("Height cannot be below 5".to_string());
            }
            if preset == GameSettingPreset::Arena && *value == 13 {
                return Some("Height cannot be below 13 in Arena mode".to_string());
            }
            if preset == GameSettingPreset::Teams && *value == 11 {
                return Some("Height cannot be below 11 in Teams mode".to_string());
            }
        } else if modif.is_positive() {
            match preset {
                GameSettingPreset::Arena => {
                    if *value == 97 {
                        return Some("Height cannot be over 97 in Arena mode".to_string());
                    }
                }
                _ => {
                    if *value == 99 {
                        return Some("Height cannot be over 99".to_string());
                    }
                }
            }
        }

        let mult = match preset {
            GameSettingPreset::Arena => 4,
            _ => 2,
        };
        *value = (*value as i16 + (modif * mult)) as u8;
        None
    }

    fn update_cheese(value: &mut u8, modif: i16) -> Option<String> {
        if *value == 0 && modif.is_negative() {
            return Some("Cheesiness cannot be below 0".to_string());
        }
        if *value == 100 && modif.is_positive() {
            return Some("Cheesiness cannot be above 100".to_string());
        }
        *value = (*value as i16 + modif) as u8;
        None
    }

    fn update_bot_count(
        preset: GameSettingPreset,
        value: &mut u8,
        player_count: u8,
        modif: i16,
    ) -> Option<String> {
        if modif.is_negative() && *value == 0 {
            return Some("Bot count cannot be below 0".to_string());
        } else if modif.is_positive() {
            match preset {
                GameSettingPreset::Corners => {
                    if *value == 4 - player_count {
                        return Some("Total players cannot exceed 4 in Corners mode".to_string());
                    }
                }
                GameSettingPreset::Arena => {
                    if *value == 10 - player_count {
                        return Some("Total players cannot exceed 10 in Arena mode".to_string());
                    }
                }
                GameSettingPreset::Teams => {
                    if *value == 8 - player_count {
                        return Some("Total players cannot exceed 8 in Teams mode".to_string());
                    }
                }
                GameSettingPreset::Custom => {
                    if *value == 99 - player_count {
                        return Some("Total players cannot exceed 99 in Custom mode".to_string());
                    }
                }
            }
        }
        *value = (*value as i16 + modif) as u8;
        if modif.is_positive() && preset == GameSettingPreset::Teams {
            return Some("Bots aren't recommended in Teams mode".to_string());
        }
        None
    }

    fn update_setting_values(&mut self, inputs: &Vec<Input>) -> Option<String> {
        if self.selected < GameSettingButtons::SettingWidth as usize
            || self.selected > GameSettingButtons::SettingBotCount as usize
        {
            return None;
        }
        let UIPage::GameSettings(settings) = &mut self.page else {
            return None;
        };

        let modif = -((inputs.menu_left() == InputState::Pressed) as i16)
            + ((inputs.menu_right() == InputState::Pressed) as i16);

        const SETTING_WIDTH_SIZE: usize = GameSettingButtons::SettingWidth as usize;
        const SETTING_HEIGHT_SIZE: usize = GameSettingButtons::SettingHeight as usize;
        const SETTING_CHEESE_SIZE: usize = GameSettingButtons::SettingCheese as usize;
        const SETTING_BOT_COUNT_SIZE: usize = GameSettingButtons::SettingBotCount as usize;

        match self.selected {
            SETTING_WIDTH_SIZE => Self::update_width(settings.preset, &mut settings.width, modif),
            SETTING_HEIGHT_SIZE => {
                Self::update_height(settings.preset, &mut settings.height, modif)
            }

            SETTING_CHEESE_SIZE => Self::update_cheese(&mut settings.cheesiness, modif),
            SETTING_BOT_COUNT_SIZE => Self::update_bot_count(
                settings.preset,
                &mut settings.bot_count,
                settings.player_count,
                modif,
            ),
            _ => None,
        }
    }

    fn tick_error(&mut self, delta: f32) {
        let UIPage::GameSettings(settings) = &mut self.page else {
            return;
        };
        settings.opacity = (settings.opacity - delta).max(0.0);
        if let Some(label) = self.canvases.last_mut() {
            if let Some(color) = &mut label.text_color {
                color.w = settings.opacity / ERROR_VISIBILITY_TIME;
            }
        }
    }

    fn set_error(&mut self, error_message: String) {
        let UIPage::GameSettings(settings) = &mut self.page else {
            return;
        };
        settings.opacity = ERROR_VISIBILITY_TIME;
        if let Some(label) = self.canvases.last_mut() {
            if let Some(text) = &mut label.text {
                *text = error_message;
            }
        }
    }

    fn update_preset(&mut self, inputs: &Vec<Input>) {
        if self.selected > GameSettingButtons::PresetCustom as usize {
            return;
        }
        if inputs.menu_left() != InputState::Pressed && inputs.menu_right() != InputState::Pressed {
            return;
        }

        let UIPage::GameSettings(settings) = &mut self.page else {
            return;
        };
        if self.selected == GameSettingPreset::Corners as usize {
            *settings = UIGameSettings {
                opacity: settings.opacity,
                ..UIGameSettings::corners(settings.player_count)
            }
        } else if self.selected == GameSettingPreset::Arena as usize {
            *settings = UIGameSettings {
                opacity: settings.opacity,
                ..UIGameSettings::arena(settings.player_count)
            }
        } else if self.selected == GameSettingPreset::Teams as usize {
            *settings = UIGameSettings {
                opacity: settings.opacity,
                ..UIGameSettings::teams(settings.player_count)
            }
        } else if self.selected == GameSettingPreset::Custom as usize {
            settings.preset = GameSettingPreset::Custom;
        }
    }

    fn create_return_value(
        &mut self,
        inputs: &Vec<Input>,
        resources: &Resources,
    ) -> (Option<AppState>, u8) {
        let UIPage::GameSettings(settings) = &mut self.page else {
            return (None, 0);
        };
        if self.selected != GameSettingButtons::Start as usize
            || inputs.menu_confirm() != InputState::Pressed
        {
            return (None, 0);
        }

        let map_settings = match settings.preset {
            GameSettingPreset::Corners => MapSettings {
                width: settings.width,
                height: settings.height,
                cheesiness: settings.cheesiness,
                spawns: settings.player_count + settings.bot_count,
                ..MapSettings::corners()
            },
            GameSettingPreset::Arena => MapSettings {
                width: settings.width,
                height: settings.height,
                cheesiness: settings.cheesiness,
                spawns: settings.player_count + settings.bot_count,
                ..MapSettings::arena()
            },
            GameSettingPreset::Teams => MapSettings {
                width: settings.width,
                height: settings.height,
                cheesiness: settings.cheesiness,
                spawns: settings.player_count + settings.bot_count,
                ..MapSettings::teams()
            },
            GameSettingPreset::Custom => MapSettings {
                width: settings.width,
                height: settings.height,
                cheesiness: settings.cheesiness,
                spawns: settings.player_count + settings.bot_count,
                ..MapSettings::default_cheese()
            },
        };

        let game_settings = crate::game::game_settings::GameSettings {
            nb_humans: settings.player_count.into(),
            map_settings,
        };

        match GameState::new_multiplayer(resources, game_settings) {
            Ok(game_state) => (Some(AppState::Game(game_state)), 1),
            Err(_) => {
                self.set_error("Map creation failed. Try reducing player/bot count.".to_string());
                (None, 0)
            }
        }
    }

    pub fn game_settings_tick(
        &mut self,
        delta: f32,
        inputs: &Vec<Input>,
        resources: &Resources,
    ) -> (Option<AppState>, u8) {
        self.button_inputs(inputs);
        if let Some(err_msg) = self.update_setting_values(inputs) {
            self.set_error(err_msg);
        }
        self.update_preset(inputs);
        let UIPage::GameSettings(settings) = &mut self.page else {
            return (None, 0);
        };
        // Makes up on Width select current preset
        self.buttons[GameSettingButtons::SettingWidth as usize]
            .neighbors
            .up = settings.preset as usize;
        self.tick_error(delta);
        self.update_label_text();
        if inputs.menu_back() == InputState::Pressed {
            return (None, 1);
        }
        self.create_return_value(inputs, resources)
    }
}
