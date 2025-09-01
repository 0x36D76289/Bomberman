use glam::{Vec2, Vec4, usize};

use crate::{
    app_state::AppState,
    game::resources::Resources,
    input::{input::Input, input_state::InputState, input_vec::MenuInput},
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
        ui_state::UIPage,
    },
};

const BACKGROUND_COLOR: Vec4 = Vec4::new(0.97, 0.88, 0.96, 1.0);

const OUTLINE_WIDTH: f32 = 0.05;

const OUTLINE_SHADE: f32 = 0.6;
const OUTLINE_COLOR: Vec4 = Vec4::new(OUTLINE_SHADE, OUTLINE_SHADE, OUTLINE_SHADE, 1.0);

const BUTTON_COLOR: Vec4 = Vec4::ONE;
const SELECTED_BUTTON_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);

const TEXT_COLOR: Vec4 = Vec4::new(0.0, 0.0, 0.0, 1.0);
const SELECTED_TEXT_COLOR: Vec4 = Vec4::ONE;

const TEXT_SIZE: f32 = 1.0;

const PRESET_BUTTON_SIZE: Vec2 = Vec2::new(0.3, 0.3);
const PRESET_GRID_COUNT: u8 = 4;
const PRESET_GRID_START_INDEX: u8 = 0;

const PRESET_VERTICAL_HEIGHT: f32 = -0.65;

const SETTINGS_SIZE: Vec2 = Vec2::new(1.0, 0.2);
const SETTING_GRID_COUNT: u8 = 8;
const SETTING_GRID_START_INDEX: u8 = 2;

const ERROR_VISIBILITY_TIME: f32 = 5.0;

const ERROR_MESSAGE_COLOR: Vec4 = Vec4::new(0.37, 0.0, 0.0, 1.0);

#[inline]
fn spread(elem_count: u8, pos: u8) -> f32 {
    ((pos + 1) as f32 / (elem_count + 1) as f32) * 2.0 - 1.0
}

fn create_outlined_button(
    pos: Vec2,
    size: Vec2,
    neighbors: ButtonNeighbors,
    canvases: &mut Vec<Canvas>,
    buttons: &mut Vec<Button>,
    text: &'static str,
) {
    canvases.push(Canvas {
        center: pos,
        width: size.x,
        height: size.y,
        color: OUTLINE_COLOR,
        texture: None,
        text: None,
        text_color: None,
        text_size: None,
    });

    buttons.push(Button {
        canvas: Canvas {
            center: pos,
            // TODO: should try to convert outline to a consistent pixel size with aspect ratio
            width: size.x - OUTLINE_WIDTH,
            height: size.y - OUTLINE_WIDTH,
            color: BUTTON_COLOR,
            texture: None,
            text: Some(text.to_string()),
            text_color: Some(TEXT_COLOR),
            text_size: Some(TEXT_SIZE),
        },
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
    Preset3,
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

        Self {
            preset: GameSettingPreset::Corners,
            width: 15,
            height: 15,
            cheesiness: 5,
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

        Self {
            preset: GameSettingPreset::Arena,
            width: 37,
            height: 21,
            cheesiness: 7,
            player_count,
            bot_count,
            opacity: 0.0,
        }
    }
    fn preset3(player_count: u8) -> Self {
        let mut bot_count = 56;

        if player_count > bot_count {
            bot_count = 0;
        } else {
            bot_count -= player_count;
        }

        Self {
            preset: GameSettingPreset::Arena,
            width: 37,
            height: 21,
            cheesiness: 7,
            player_count,
            bot_count,
            opacity: 0.0,
        }
    }
}

enum GameSettingButtons {
    PresetCorners,
    PresetArena,
    Preset3,
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
                x: spread(PRESET_GRID_COUNT, PRESET_GRID_START_INDEX + 0),
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::PresetCorners as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::PresetCorners as usize,
                right: GameSettingButtons::PresetArena as usize,
            },
            &mut canvases,
            &mut buttons,
            "Corners",
        );
        buttons[0].toggle();

        // PresetArena
        create_outlined_button(
            Vec2 {
                x: spread(PRESET_GRID_COUNT, PRESET_GRID_START_INDEX + 1),
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::PresetArena as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::PresetCorners as usize,
                right: GameSettingButtons::Preset3 as usize,
            },
            &mut canvases,
            &mut buttons,
            "Arena",
        );

        // Preset3
        create_outlined_button(
            Vec2 {
                x: spread(PRESET_GRID_COUNT, PRESET_GRID_START_INDEX + 2),
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::Preset3 as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::PresetArena as usize,
                right: GameSettingButtons::PresetCustom as usize,
            },
            &mut canvases,
            &mut buttons,
            "Preset 3",
        );

        // Custom
        create_outlined_button(
            Vec2 {
                x: spread(PRESET_GRID_COUNT, PRESET_GRID_START_INDEX + 3),
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::PresetCustom as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::Preset3 as usize,
                right: GameSettingButtons::PresetCustom as usize,
            },
            &mut canvases,
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
            &mut canvases,
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
            &mut canvases,
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
            &mut canvases,
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
            &mut canvases,
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
            &mut canvases,
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
        let UIPage::GameSettings(settings) = &mut self.page else {
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
        // INFO: error detect here..?
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
            SETTING_WIDTH_SIZE => {
                if modif.is_negative() && settings.width == 5 {
                    return Some("Width cannot be below 5".to_string());
                } else if modif.is_positive() && settings.width == 99 {
                    return Some("Width cannot be over 99".to_string());
                }
                // TODO: change with presets
                settings.width = (settings.width as i16 + modif * 2) as u8;
            }
            SETTING_HEIGHT_SIZE => {
                if modif.is_negative() && settings.height == 5 {
                    return Some("Height cannot be below 5".to_string());
                } else if modif.is_positive() && settings.height == 99 {
                    return Some("Height cannot be over 99".to_string());
                }
                settings.height = (settings.height as i16 + modif * 2) as u8;
            }
            SETTING_CHEESE_SIZE => {
                if modif.is_negative() && settings.cheesiness == 0 {
                    return Some("Cheesiness cannot be below 0".to_string());
                } else if modif.is_positive() && settings.cheesiness == 100 {
                    return Some("Cheesiness cannot be above 100".to_string());
                }
                // TODO: Max
                settings.cheesiness = (settings.cheesiness as i16 + modif) as u8;
            }
            SETTING_BOT_COUNT_SIZE => {
                if modif.is_negative() && settings.bot_count == 0 {
                    return Some("Bot count cannot be below 0".to_string());
                } else if modif.is_positive() {
                    if settings.preset == GameSettingPreset::Corners
                        && settings.bot_count == 4 - settings.player_count
                    {
                        return Some("Total players cannot exceed 4 in Corners mode".to_string());
                    } else if settings.preset == GameSettingPreset::Arena
                        && settings.bot_count == 10 - settings.player_count
                    {
                        return Some("Total players cannot exceed 8 in Arena mode".to_string());
                    }
                }
                settings.bot_count = (settings.bot_count as i16 + modif) as u8;
            }
            _ => (),
        };
        return None;
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
        } else if self.selected == GameSettingPreset::Preset3 as usize {
            *settings = UIGameSettings {
                opacity: settings.opacity,
                ..UIGameSettings::preset3(settings.player_count)
            }
        } else if self.selected == GameSettingPreset::Custom as usize {
            settings.preset = GameSettingPreset::Custom;
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
        //       use vec::last in controls for selecting error
        // if selected is a preset then update with preset settings
        // change settings text content to match
        // update alpha
        let UIPage::GameSettings(settings) = &mut self.page else {
            return (None, 0);
        };
        self.buttons[GameSettingButtons::SettingWidth as usize]
            .neighbors
            .up = settings.preset as usize;
        settings.opacity = (settings.opacity - delta).max(0.0);
        self.tick_error(delta);
        self.update_label_text();
        (None, 0)
    }
}
