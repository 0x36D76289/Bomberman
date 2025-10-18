use glam::Vec2;

use crate::{
    app_state::AppState,
    input::{
        controller::{create_bind, is_bindable_action},
        event::InputEvent,
        input::{BIND_LEN, Input},
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
    },
};

enum BindsButtons {
    Up,
    Down,
    Left,
    Right,
    Bomb,
    Back,
    Delete,
    Done,
}

fn create_button() -> Button {
    const BUTTON_HEIGHT: f32 = 0.2;
    const BUTTON_WIDTH: f32 = 0.3;
    Button {
        canvas: Canvas {
            width: BUTTON_WIDTH,
            height: BUTTON_HEIGHT,
            color: BUTTON_COLOR,
            text_color: Some(TEXT_COLOR),
            text_size: Some(TEXT_SIZE * 2.0 / 3.0),
            ..Default::default()
        },
        outline_color: Some(OUTLINE_COLOR),
        selected_color: SELECTED_BUTTON_COLOR,
        selected_text_color: Some(SELECTED_TEXT_COLOR),
        ..Default::default()
    }
}

impl UiState {
    pub fn binds(player: usize, ratio: f32) -> Self {
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

        // Player <x> canvas
        canvases.push(Canvas {
            center: Vec2 {
                x: 0.0,
                y: -1.0 + OFFSET_DIR,
            },
            width: 0.0,
            height: 0.0,
            color: TEXT_COLOR,
            texture: None,
            text: Some(format!("Player {player} Binds")),
            text_color: Some(TEXT_COLOR),
            text_size: Some(TEXT_SIZE * 2.0),
        });

        const CENTER_DIST: f32 = 0.4;
        const OFFSET_DIR: f32 = 0.3;
        const OFFSET_BUTT: f32 = 0.2;

        let mut buttons = Vec::new();
        // up
        buttons.push(
            create_button()
                .with_pos(Vec2 {
                    x: -CENTER_DIST,
                    y: -OFFSET_DIR,
                })
                .with_neighbors(ButtonNeighbors {
                    up: BindsButtons::Up as usize,
                    down: BindsButtons::Down as usize,
                    left: BindsButtons::Left as usize,
                    right: BindsButtons::Right as usize,
                }),
        );
        // down
        buttons.push(
            create_button()
                .with_pos(Vec2 {
                    x: -CENTER_DIST,
                    y: OFFSET_DIR,
                })
                .with_neighbors(ButtonNeighbors {
                    up: BindsButtons::Up as usize,
                    down: BindsButtons::Delete as usize,
                    left: BindsButtons::Left as usize,
                    right: BindsButtons::Right as usize,
                }),
        );
        // left
        buttons.push(
            create_button()
                .with_pos(Vec2 {
                    x: -CENTER_DIST - OFFSET_DIR * (1.0 / ratio),
                    y: 0.0,
                })
                .with_neighbors(ButtonNeighbors {
                    up: BindsButtons::Up as usize,
                    down: BindsButtons::Down as usize,
                    left: BindsButtons::Left as usize,
                    right: BindsButtons::Right as usize,
                }),
        );
        // right
        buttons.push(
            create_button()
                .with_pos(Vec2 {
                    x: -CENTER_DIST + OFFSET_DIR * (1.0 / ratio),
                    y: 0.0,
                })
                .with_neighbors(ButtonNeighbors {
                    up: BindsButtons::Up as usize,
                    down: BindsButtons::Down as usize,
                    left: BindsButtons::Left as usize,
                    right: BindsButtons::Bomb as usize,
                }),
        );
        // Bomb
        buttons.push(
            create_button()
                .with_pos(Vec2 {
                    x: CENTER_DIST - OFFSET_BUTT,
                    y: 0.0,
                })
                .with_neighbors(ButtonNeighbors {
                    up: BindsButtons::Bomb as usize,
                    down: BindsButtons::Done as usize,
                    left: BindsButtons::Right as usize,
                    right: BindsButtons::Back as usize,
                }),
        );
        // Back
        buttons.push(
            create_button()
                .with_pos(Vec2 {
                    x: CENTER_DIST + OFFSET_BUTT,
                    y: 0.0,
                })
                .with_neighbors(ButtonNeighbors {
                    up: BindsButtons::Back as usize,
                    down: BindsButtons::Done as usize,
                    left: BindsButtons::Bomb as usize,
                    right: BindsButtons::Back as usize,
                }),
        );
        // Delete
        buttons.push(
            create_button()
                .with_pos(Vec2 {
                    x: -OFFSET_BUTT,
                    y: 1.0 - OFFSET_DIR,
                })
                .with_neighbors(ButtonNeighbors {
                    up: BindsButtons::Down as usize,
                    down: BindsButtons::Delete as usize,
                    left: BindsButtons::Delete as usize,
                    right: BindsButtons::Done as usize,
                }),
        );
        // Done
        buttons.push(
            create_button()
                .with_pos(Vec2 {
                    x: OFFSET_BUTT,
                    y: 1.0 - OFFSET_DIR,
                })
                .with_neighbors(ButtonNeighbors {
                    up: BindsButtons::Bomb as usize,
                    down: BindsButtons::Done as usize,
                    left: BindsButtons::Delete as usize,
                    right: BindsButtons::Done as usize,
                }),
        );
        buttons[BindsButtons::Done as usize].canvas.text = Some("Done".to_string());

        buttons[0].toggle();
        Self {
            canvases,
            buttons,
            selected: 0,
            render_info: Default::default(),
        }
    }

    fn set_names(&mut self, waiting: isize, settings: &Settings, player: usize) {
        self.buttons[BindsButtons::Up as usize].canvas.text = Some("Up".to_string());
        self.buttons[BindsButtons::Down as usize].canvas.text = Some("Down".to_string());
        self.buttons[BindsButtons::Left as usize].canvas.text = Some("Left".to_string());
        self.buttons[BindsButtons::Right as usize].canvas.text = Some("Right".to_string());
        self.buttons[BindsButtons::Bomb as usize].canvas.text = Some("Bomb".to_string());
        self.buttons[BindsButtons::Delete as usize].canvas.text = Some("Delete".to_string());
        self.buttons[BindsButtons::Back as usize].canvas.text = Some("Back".to_string());

        if self.selected < BIND_LEN {
            let bind = settings.binds[player][self.selected];
            self.buttons[self.selected].canvas.text = Some(format!("{bind}"));
        }
        if waiting >= 0 {
            self.buttons[waiting as usize].canvas.text = Some("...".to_string());
        }
    }

    fn set_wait(&mut self, inputs: &Vec<Input>, waiting: &mut isize) {
        // TODO: take events and check if click
        if self.selected == BindsButtons::Delete as usize
            || self.selected == BindsButtons::Done as usize
        {
            return;
        }
        if !(inputs.menu_confirm() == InputState::Pressed) {
            return;
        }
        // here a bind is selected and someone is changing it
        *waiting = self.selected as isize;
    }

    pub fn set_bind(
        &mut self,
        events: &Vec<InputEvent>,
        settings: &mut Settings,
        player: &mut usize,
        waiting: &mut isize,
    ) {
        for event in events {
            if is_bindable_action(event) {
                settings.binds[*player][*waiting as usize] = create_bind(event);
                settings.save();
                *waiting = -1;
                return;
            }
        }
    }

    pub fn binds_tick(
        &mut self,
        inputs: &Vec<Input>,
        events: &Vec<InputEvent>,
        settings: &mut Settings,
        player: &mut usize,
        waiting: &mut isize,
    ) -> (Option<AppState>, u8) {
        if *waiting < 0 {
            self.button_inputs(inputs);
            self.set_wait(inputs, waiting);
        } else {
            self.set_bind(events, settings, player, waiting);
        }
        self.set_names(*waiting, settings, *player);
        if inputs.menu_confirm() == InputState::Pressed {
            if self.selected == BindsButtons::Delete as usize {
                settings.binds.remove(*player);
                settings.save();
            }
            if self.selected >= BindsButtons::Delete as usize {
                return (None, 1);
            }
        }
        if inputs.menu_back() == InputState::Pressed {
            return (None, 1);
        }
        (None, 0)
    }
}
