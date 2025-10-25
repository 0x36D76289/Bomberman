use glam::Vec2;

use crate::{
    app_state::AppState,
    input::{
        input::{Input, default_binds},
        input_state::InputState,
        input_vec::MenuInput,
    },
    settings::settings::Settings,
    ui::{
        UiState,
        button::{Button, ButtonNeighbors},
        canvas::Canvas,
        consts::{
            BACKGROUND_COLOR, BUTTON_COLOR, OUTLINE_COLOR, SELECTED_BUTTON_COLOR,
            SELECTED_TEXT_COLOR, TEXT_COLOR, TEXT_SIZE,
        },
        utils::spread,
    },
};

/// The ordered list of buttons in the settings ui page
enum SettingsButtons {
    Resolution,
    Filtering,
    Fullscreen,
    Framecap,
    MusicVol,
    SfxVol,
    Binds,
}

/// The amount of values in the enum, used for layout
const BUTTON_COUNT: u8 = 7;

/// The shortcut to unified button creation on the settings page
fn create_button(index: u8, text: &str) -> Button {
    Button {
        canvas: Canvas {
            center: Vec2 {
                x: 0.0,
                y: spread(BUTTON_COUNT + 2, index + 2),
            },
            width: 1.0,
            height: 2.0 / (BUTTON_COUNT + 2) as f32,
            color: BUTTON_COLOR,
            texture: None,
            text: Some(text.to_string()),
            text_color: Some(TEXT_COLOR),
            text_size: Some(TEXT_SIZE),
        },
        outline_color: Some(OUTLINE_COLOR),
        neighbors: ButtonNeighbors {
            up: ((index + BUTTON_COUNT - 1) % BUTTON_COUNT) as usize,
            down: ((index + 1) % BUTTON_COUNT) as usize,
            left: index as usize,
            right: index as usize,
        },
        selected_color: SELECTED_BUTTON_COLOR,
        selected_text_color: Some(SELECTED_TEXT_COLOR),
        selected_texture: None,
    }
}

impl UiState {
    /// The general settings ui page constructor
    pub fn settings() -> Self {
        let mut canvases = Vec::new();

        canvases.push(Canvas {
            center: Vec2 { x: 0.0, y: 0.0 },
            width: 2.0,
            height: 2.0,
            color: BACKGROUND_COLOR,
            texture: None,
            text: None,
            text_color: None,
            text_size: None,
        });

        canvases.push(Canvas {
            center: Vec2 {
                x: 0.0,
                y: spread(BUTTON_COUNT + 2, 0),
            },
            width: 0.0,
            height: 0.0,
            color: TEXT_COLOR,
            texture: None,
            text: Some("Settings".to_string()),
            text_color: Some(TEXT_COLOR),
            text_size: Some(TEXT_SIZE * 2.0),
        });

        let mut buttons = Vec::new();

        buttons.push(create_button(0, "Resolution"));
        buttons.push(create_button(1, "Filtering"));
        buttons.push(create_button(2, "Fullscreen"));
        buttons.push(create_button(3, "Framerate Limit"));
        buttons.push(create_button(4, "Music Volume"));
        buttons.push(create_button(5, "SFX Volume"));
        buttons.push(create_button(6, "Binds"));
        buttons[0].toggle();

        Self {
            canvases,
            buttons,
            selected: 0,
            render_info: Default::default(),
        }
    }

    /// Updates the resolution from a preset list
    fn update_resolution(&mut self, settings: &mut Settings, direction: i8) {
        let index = SettingsButtons::Resolution as usize;
        if self.selected != index {
            self.buttons[index].canvas.text = Some("Resolution".to_string());
            return;
        }

        self.buttons[index].canvas.text = Some(format!(
            "{}x{}",
            settings.resolution.0, settings.resolution.1
        ));
        if direction == 0 {
            return;
        }

        const RESOLUTION_COUNT: usize = 17;
        const RESOLUTIONS: [(u32, u32); RESOLUTION_COUNT] = [
            (160, 120),
            (256, 192),
            (320, 240),
            (400, 300),
            (512, 384),
            (640, 480),
            (800, 600),
            (960, 720),
            (1024, 768),
            (1280, 960),
            (1600, 1200),
            (1920, 1440),
            (2560, 1920),
            (2880, 2160),
            (3200, 2400),
            (4096, 3072),
            (6400, 4800),
        ];

        let res_index = RESOLUTIONS.binary_search(&settings.resolution);
        if res_index.is_err() {
            println!("err");
            settings.resolution = Settings::default().resolution;
            settings.save();
            return;
        }

        let res_index = res_index.unwrap();
        let res_index =
            (res_index as isize + direction as isize).clamp(0, RESOLUTION_COUNT as isize - 1);
        settings.resolution = RESOLUTIONS[res_index as usize];
        settings.save();
    }

    /// Updates the filtering of the game
    fn update_filtering(&mut self, settings: &mut Settings, direction: i8) {
        let index = SettingsButtons::Filtering as usize;
        if self.selected != index {
            self.buttons[index].canvas.text = Some("Filtering".to_string());
            return;
        }
        self.buttons[index].canvas.text = Some(format!("{}", settings.filtering));
        if direction != 0 {
            settings.filtering = !settings.filtering;
            settings.save();
        }
    }

    /// Updates the fullscreen display of the game
    fn update_fullscreen(&mut self, settings: &mut Settings, direction: i8) {
        let index = SettingsButtons::Fullscreen as usize;
        if self.selected != index {
            self.buttons[index].canvas.text = Some("Fullscreen".to_string());
            return;
        }
        self.buttons[index].canvas.text = Some(format!("{}", settings.fullscreen));
        if direction != 0 {
            settings.fullscreen = !settings.fullscreen;
            settings.save();
        }
    }

    /// Updates the maximum framerate from a preset list of values + vsync and uncapped
    fn update_framerate(&mut self, settings: &mut Settings, direction: i8) {
        let index = SettingsButtons::Framecap as usize;
        if self.selected != index {
            self.buttons[index].canvas.text = Some("Framerate Limit".to_string());
            return;
        }

        //render
        let val = settings.framerate_limit;
        self.buttons[index].canvas.text = Some(if val < 0 {
            "Uncapped".to_string()
        } else if val == 0 {
            "VSync".to_string()
        } else {
            format!("{}", settings.framerate_limit)
        });

        if direction == 0 {
            return;
        }

        // find pos
        const FRAMECAP_COUNT: usize = 10;
        const FRAMECAPS: [i32; FRAMECAP_COUNT] = [0, 24, 30, 60, 90, 120, 144, 240, 360, -1];
        let sup_index = if val < 0 {
            FRAMECAP_COUNT - 1
        } else if val > FRAMECAPS[FRAMECAP_COUNT - 2] {
            FRAMECAP_COUNT - 1
        } else {
            let mut ret = 0;
            for i in 0..(FRAMECAP_COUNT - 1) {
                ret = i;
                if val <= FRAMECAPS[i] {
                    break;
                }
            }
            ret
        };

        // save new
        let new = (sup_index as isize + direction as isize).rem_euclid(FRAMECAP_COUNT as isize);
        settings.framerate_limit = FRAMECAPS[new as usize];
        settings.save();
    }

    /// Updates the music volume
    fn update_music_volume(&mut self, settings: &mut Settings, direction: i8) {
        let index = SettingsButtons::MusicVol as usize;
        if self.selected != index {
            self.buttons[index].canvas.text = Some("Music Volume".to_string());
            return;
        }

        const TICKS: usize = 21;
        let mut str: String = "-".to_string().repeat(TICKS);
        let vol = (settings.volume_music.clamp(0.0, 1.0) * (TICKS - 1) as f32) as usize;
        str.replace_range(vol..=vol, "|");
        self.buttons[index].canvas.text = Some(str);

        if direction != 0 {
            settings.volume_music += direction as f32 * (1.0 / (TICKS as f32 - 1.0));
            settings.volume_music = settings.volume_music.clamp(0.0, 1.0);
            settings.save();
        }
    }

    /// Updates the Sound Effects volume
    fn update_sfx_volume(&mut self, settings: &mut Settings, direction: i8) {
        let index = SettingsButtons::SfxVol as usize;
        if self.selected != index {
            self.buttons[index].canvas.text = Some("SFX Volume".to_string());
            return;
        }

        // TODO: play a sound effect on change (explosion)
        const TICKS: usize = 21;
        let mut str: String = "-".to_string().repeat(TICKS);
        let vol = (settings.volume_sfx.clamp(0.0, 1.0) * (TICKS - 1) as f32) as usize;
        str.replace_range(vol..=vol, "|");
        self.buttons[index].canvas.text = Some(str);

        if direction != 0 {
            settings.volume_sfx += direction as f32 * (1.0 / (TICKS as f32 - 1.0));
            settings.volume_sfx = settings.volume_sfx.clamp(0.0, 1.0);
            settings.save();
        }
    }

    /// Changes the selected player in binds and opens the ui page if selected
    fn update_binds(
        &mut self,
        settings: &mut Settings,
        selected_player: &mut usize,
        direction: i8,
        open: bool,
        ratio: f32,
    ) -> Option<(Option<AppState>, u8)> {
        let index = SettingsButtons::Binds as usize;
        if self.selected != index {
            self.buttons[index].canvas.text = Some("Binds".to_string());
            return None;
        }

        let num = selected_player;

        let existing_count = settings.binds.len();
        *num =
            (*num as isize + direction as isize).rem_euclid(existing_count as isize + 1) as usize;
        self.buttons[index].canvas.text = Some(if *num != existing_count {
            format!("Player {}", *num + 1)
        } else {
            format!("Player {} (+)", *num + 1)
        });

        if open {
            if *num == existing_count {
                settings.binds.push(default_binds());
                settings.save();
            }
            return Some((Some(AppState::binds(*num, ratio)), 0));
        }
        // if input create binds page with player count arg
        // binds has keys, inputs, events, settings?
        return None;
    }

    /// The general settings ui page tick function
    pub fn settings_tick(
        &mut self,
        inputs: &Vec<Input>,
        settings: &mut Settings,
        selected_player: &mut usize,
        ratio: f32,
    ) -> (Option<AppState>, u8) {
        self.button_inputs(inputs);

        let mut direction: i8 = 0;
        if inputs.menu_left() == InputState::Pressed {
            direction -= 1;
        }
        if inputs.menu_right() == InputState::Pressed {
            direction += 1;
        }

        self.update_resolution(settings, direction);
        self.update_filtering(settings, direction);
        self.update_fullscreen(settings, direction);
        self.update_framerate(settings, direction);
        self.update_music_volume(settings, direction);
        self.update_sfx_volume(settings, direction);
        self.update_sfx_volume(settings, direction);
        if let Some(ret) = self.update_binds(
            settings,
            selected_player,
            direction,
            inputs.menu_confirm() == InputState::Pressed,
            ratio,
        ) {
            return ret;
        }

        if inputs.menu_back() == InputState::Pressed {
            return (None, 1);
        }
        (None, 0)
    }
}
