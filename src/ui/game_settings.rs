use std::ops::Deref;

use glam::{Vec2, Vec4};

use crate::{
    app_state::AppState,
    game::{
        player,
        resources::{self, Resources},
    },
    input::input::Input,
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
            //TODO: should try to convert outline to a consistent pixel size with aspect ratio
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

#[derive(Debug, Copy, Clone)]
pub struct UIGameSettings {
    preset: u8,
    width: u8,
    height: u8,
    cheesiness: u8,
    player_count: u8,
    bot_count: u8,
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
            preset: 0,
            width: 15,
            height: 15,
            cheesiness: 5,
            player_count,
            bot_count,
            opacity: 0.0,
        }
    }
}

enum GameSettingButtons {
    PresetCorners,
    Preset2,
    Preset3,
    PresetCustom,
    SettingWidth,
    SettingHeight,
    SettingCheese,
    SettingBotCount,
    Start,
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
                right: GameSettingButtons::Preset2 as usize,
            },
            &mut canvases,
            &mut buttons,
            "Corners",
        );
        buttons[0].toggle();

        // Preset2
        create_outlined_button(
            Vec2 {
                x: spread(PRESET_GRID_COUNT, PRESET_GRID_START_INDEX + 1),
                y: PRESET_VERTICAL_HEIGHT,
            },
            PRESET_BUTTON_SIZE,
            ButtonNeighbors {
                up: GameSettingButtons::Preset2 as usize,
                down: GameSettingButtons::SettingWidth as usize,
                left: GameSettingButtons::PresetCorners as usize,
                right: GameSettingButtons::Preset3 as usize,
            },
            &mut canvases,
            &mut buttons,
            "Preset 2",
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
                left: GameSettingButtons::Preset2 as usize,
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

        const SETTINGS_SIZE: Vec2 = Vec2::new(1.0, 0.2);
        const SETTING_GRID_COUNT: u8 = 8;
        const SETTING_GRID_START_INDEX: u8 = 2;
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

        // TODO: add labels for values
        // labels are canvas items
        // access with canvases[buttons.len + 1/2/3/4]

        Self {
            canvases,
            buttons,
            is_transparent: false,
            selected: 0,
            page: UIPage::GameSettings(UIGameSettings::corners(player_count)),
        }
    }

    pub fn game_settings_tick(
        &mut self,
        inputs: &Vec<Input>,
        resources: &Resources,
    ) -> (Option<AppState>, u8) {
        self.button_inputs(inputs);
        let UIPage::GameSettings(settings) = &mut self.page else {
            return (None, 0);
        };
        //       use vec::last in controls
        // if selected is a preset then update with preset settings
        // if selected is a setting then update
        // change settings text content to match
        // update alpha
        self.buttons[GameSettingButtons::SettingWidth as usize]
            .neighbors
            .up = settings.preset as usize;
        (None, 0)
    }
}
