use crate::input::{event::InputEvent, input::Binds};
use gilrs::EventType;

const DEADZONE: f32 = 0.2;

pub fn is_bindable_action(event: &InputEvent) -> bool {
    match event {
        InputEvent::Keyboard { key: _, down } => *down,
        InputEvent::ControllerInput {
            controller: _,
            event,
        } => match event {
            EventType::ButtonChanged(_, value, _) => *value > DEADZONE,
            EventType::AxisChanged(_, value, _) => value.abs() > DEADZONE,
            _ => false,
        },
    }
}

pub fn create_bind(input_event: &InputEvent) -> InputEvent {
    match input_event {
        InputEvent::ControllerInput { controller, event } => match event {
            EventType::ButtonChanged(button, _, code) => InputEvent::ControllerInput {
                controller: *controller,
                event: EventType::ButtonChanged(*button, 1.0, *code),
            },
            EventType::AxisChanged(axis, value, code) => InputEvent::ControllerInput {
                controller: *controller,
                event: EventType::AxisChanged(*axis, value.signum(), *code),
            },
            _ => *input_event,
        },
        _ => *input_event,
    }
}

fn normalize_value(value: f32) -> f32 {
    if value.abs() < DEADZONE {
        return 0.0;
    }
    value.signum()
}

pub fn input_events_compare(test: &InputEvent, predicate: &InputEvent) -> Option<bool> {
    if *test == *predicate {
        return Some(true);
    }
    if let InputEvent::ControllerInput {
        controller: test_controller,
        event: test_event,
    } = test
        && let InputEvent::ControllerInput {
            controller: predicate_controller,
            event: predicate_event,
        } = predicate
    {
        if *test_controller != *predicate_controller {
            return None;
        }

        if let EventType::ButtonChanged(test_button, test_value, _) = test_event
            && let EventType::ButtonChanged(predicate_button, predicate_value, _) = predicate_event
        {
            if test_button != predicate_button {
                return None;
            }
            return Some(normalize_value(*test_value) == normalize_value(*predicate_value));
        }

        if let EventType::AxisChanged(test_axis, test_value, _) = test_event
            && let EventType::AxisChanged(predicate_axis, predicate_value, _) = predicate_event
        {
            if test_axis != predicate_axis {
                return None;
            }
            return Some(normalize_value(*test_value) == normalize_value(*predicate_value));
        }
    }
    if let InputEvent::Keyboard {
        key: test_key,
        down: test_down,
    } = test
        && let InputEvent::Keyboard {
            key: predicate_key,
            down: predicate_down,
        } = predicate
    {
        if *test_key != *predicate_key {
            return None;
        }
        return Some(*test_down == *predicate_down);
    }
    None
}

pub fn uses_controller(binds: &Binds) -> bool {
    binds
        .iter()
        .any(|bind| matches!(bind, InputEvent::ControllerInput { .. }))
}
