use pyo3::prelude::*;

use crate::*;

#[derive(Clone, Debug)]
pub enum RuntimeKeyActionSequenceAction {
    KeyAction(KeyActionWithMods),
    SyncModifiers,
}

#[derive(Clone, Debug)]
pub enum RuntimeAction {
    ActionSequence(Vec<KeyActionWithMods>),
    // flags are used to release modifiers in the trigger
    PythonCallback(Arc<PyObject>),
    NOP,
}

pub type Mapping = (KeyActionWithMods, RuntimeAction);
pub type Mappings = HashMap<KeyActionWithMods, RuntimeAction>;

pub fn map_action_to_seq(from: KeyActionWithMods, to: Vec<ParsedKeyAction>) -> Mapping {
    (from, RuntimeAction::ActionSequence(to.to_key_actions_with_mods()))
}

pub fn map_action_to_click(from: &KeyActionWithMods, to: &KeyClickActionWithMods) -> Mapping {
    (
        from.clone(),
        RuntimeAction::ActionSequence(vec![
            KeyActionWithMods::new(to.key, TYPE_DOWN, to.modifiers),
            KeyActionWithMods::new(to.key, TYPE_UP, to.modifiers),
        ]),
    )
}

pub fn map_action_to_action(from: &KeyActionWithMods, to: &KeyActionWithMods) -> Mapping {
    (from.clone(), RuntimeAction::ActionSequence(vec![*to]))
}

pub fn map_click_to_seq(from: KeyClickActionWithMods, to: Vec<ParsedKeyAction>) -> [Mapping; 3] {
    let down_mapping = (
        KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() },
        RuntimeAction::ActionSequence(to.to_key_actions_with_mods()),
    );

    // stub up and repeat, click only triggers sequence on down press
    let up_mapping =
        (KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    let repeat_mapping = (
        KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() },
        RuntimeAction::NOP,
    );
    [down_mapping, up_mapping, repeat_mapping]
}

pub fn map_click_to_click(from: &KeyClickActionWithMods, to: &KeyClickActionWithMods) -> [Mapping; 3] {
    [
        (
            KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() },
            RuntimeAction::ActionSequence(vec![KeyActionWithMods::new(to.key, TYPE_DOWN, to.modifiers)]),
        ),
        (
            KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() },
            RuntimeAction::ActionSequence(vec![KeyActionWithMods::new(to.key, TYPE_REPEAT, to.modifiers)]),
        ),
        (
            KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() },
            RuntimeAction::ActionSequence(vec![KeyActionWithMods::new(to.key, TYPE_UP, to.modifiers)]),
        ),
    ]
}

pub fn map_click_to_action(from: &KeyClickActionWithMods, to: &KeyActionWithMods) -> [Mapping; 3] {
    [
        (
            KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() },
            RuntimeAction::ActionSequence(vec![KeyActionWithMods::new(to.key, TYPE_DOWN, to.modifiers)]),
        ),
        // stub up and repeat, click only triggers action on down press
        (KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, RuntimeAction::NOP),
        (
            KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() },
            RuntimeAction::NOP,
        ),
    ]
}

pub fn release_restore_modifiers(from_flags: &KeyModifierFlags, to_flags: &KeyModifierFlags) -> Vec<KeyAction> {
    let mut output_events = vec![];

    // first release, then press
    for value in [0, 1] {
        for (from, to, key) in [
            (from_flags.left_ctrl, to_flags.left_ctrl, KEY_LEFTCTRL),
            (from_flags.right_ctrl, to_flags.right_ctrl, KEY_RIGHTCTRL),
            (from_flags.left_shift, to_flags.left_shift, KEY_LEFTSHIFT),
            (from_flags.right_shift, to_flags.right_shift, KEY_RIGHTSHIFT),
            (from_flags.left_alt, to_flags.left_alt, KEY_LEFTALT),
            (from_flags.right_alt, to_flags.right_alt, KEY_RIGHTALT),
            (from_flags.left_meta, to_flags.left_meta, KEY_LEFTMETA),
            (from_flags.right_meta, to_flags.right_meta, KEY_RIGHTMETA),
        ] {
            if value == 0 && from && !to {
                output_events.push(KeyAction { key: key.into(), value });
            }
            if value == 1 && !from && to {
                output_events.push(KeyAction { key: key.into(), value });
            }
        }
    }

    if output_events.len() > 0 {
        // output_events.push(SYN_REPORT.clone());
        output_events.push(KeyAction::new(
            Key { event_code: evdev_rs::enums::EventCode::EV_SYN(evdev_rs::enums::EV_SYN::SYN_REPORT) },
            0,
        ));
    }

    output_events
}
