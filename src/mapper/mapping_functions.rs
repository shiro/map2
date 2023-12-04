use pyo3::prelude::*;

use crate::*;

#[derive(Clone, Debug)]
pub enum RuntimeKeyAction {
    KeyAction(KeyAction),
    ReleaseRestoreModifiers(KeyModifierFlags, KeyModifierFlags, i32),
}

#[derive(Clone, Debug)]
pub enum RuntimeAction {
    ActionSequence(Vec<RuntimeKeyAction>),
    // flags are used to release modifiers in the trigger
    PythonCallback(KeyModifierFlags, PyObject),
    NOP,
}

pub type Mapping = (KeyActionWithMods, RuntimeAction);
pub type Mappings = HashMap<KeyActionWithMods, RuntimeAction>;


pub fn map_action_to_seq(from: KeyActionWithMods, to: Vec<ParsedKeyAction>) -> Mapping {
    let mut seq: Vec<RuntimeKeyAction> = to.to_key_actions()
        .into_iter()
        .map(|action| RuntimeKeyAction::KeyAction(action))
        .collect();

    seq.insert(0, RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), KeyModifierFlags::new(), TYPE_UP));

    (from, RuntimeAction::ActionSequence(seq))
}

pub fn map_action_to_click(from: &KeyActionWithMods, to: &KeyClickActionWithMods) -> Mapping {
    let mut seq = vec![];
    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTCTRL.into(), value: TYPE_DOWN })); }
    if !from.modifiers.alt && to.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTALT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.right_alt && to.modifiers.right_alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_RIGHTALT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.shift && to.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTSHIFT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.meta && to.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTMETA.into(), value: TYPE_DOWN })); }

    seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));
    seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));

    // revert to original
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTCTRL.into(), value: TYPE_UP })); }
    if !from.modifiers.alt && to.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTALT.into(), value: TYPE_UP })); }
    if !from.modifiers.right_alt && to.modifiers.right_alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_RIGHTALT.into(), value: TYPE_UP })); }
    if !from.modifiers.shift && to.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTSHIFT.into(), value: TYPE_UP })); }
    if !from.modifiers.meta && to.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTMETA.into(), value: TYPE_UP })); }

    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

    (from.clone(), RuntimeAction::ActionSequence(seq))
}

pub fn map_action_to_action(from: &KeyActionWithMods, to: &KeyActionWithMods) -> Mapping {
    let mut seq = vec![];
    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTCTRL.into(), value: TYPE_DOWN })); }
    if !from.modifiers.alt && to.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTALT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.right_alt && to.modifiers.right_alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_RIGHTALT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.shift && to.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTSHIFT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.meta && to.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTMETA.into(), value: TYPE_DOWN })); }

    seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: to.value }));

    // revert to original
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTCTRL.into(), value: TYPE_UP })); }
    if !from.modifiers.alt && to.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTALT.into(), value: TYPE_UP })); }
    if !from.modifiers.right_alt && to.modifiers.right_alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_RIGHTALT.into(), value: TYPE_UP })); }
    if !from.modifiers.shift && to.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTSHIFT.into(), value: TYPE_UP })); }
    if !from.modifiers.meta && to.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTMETA.into(), value: TYPE_UP })); }

    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

    (from.clone(), RuntimeAction::ActionSequence(seq))
}

pub fn map_click_to_seq(from: KeyClickActionWithMods, to: Vec<ParsedKeyAction>) -> [Mapping; 3] {
    let mut seq: Vec<RuntimeKeyAction> = to.to_key_actions()
        .into_iter()
        .map(|action| RuntimeKeyAction::KeyAction(action))
        .collect();

    seq.insert(0, RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), KeyModifierFlags::new(), TYPE_UP));

    let down_mapping = (KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    // stub up and repeat, click only triggers sequence on down press
    let up_mapping = (KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    let repeat_mapping = (KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    [down_mapping, up_mapping, repeat_mapping]
}

pub fn map_click_to_click(from: &KeyClickActionWithMods, to: &KeyClickActionWithMods) -> [Mapping; 3] {
    let down_mapping;
    {
        let mut seq = vec![];
        seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTCTRL.into(), value: TYPE_DOWN })); }
        if to.modifiers.alt && !from.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTALT.into(), value: TYPE_DOWN })); }
        if to.modifiers.right_alt && !from.modifiers.right_alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_RIGHTALT.into(), value: TYPE_DOWN })); }
        if to.modifiers.shift && !from.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTSHIFT.into(), value: TYPE_DOWN })); }
        if to.modifiers.meta && !from.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTMETA.into(), value: TYPE_DOWN })); }
        seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));
        down_mapping = (KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    }
    let up_mapping;
    {
        let mut seq = vec![];
        seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));
        if to.modifiers.ctrl && !from.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTCTRL.into(), value: TYPE_UP })); }
        if to.modifiers.alt && !from.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTALT.into(), value: TYPE_UP })); }
        if to.modifiers.right_alt && !from.modifiers.right_alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_RIGHTALT.into(), value: TYPE_UP })); }
        if to.modifiers.shift && !from.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTSHIFT.into(), value: TYPE_UP })); }
        if to.modifiers.meta && !from.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTMETA.into(), value: TYPE_UP })); }
        seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));
        up_mapping = (KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    }
    let repeat_mapping;
    {
        let mut seq = vec![];
        seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: TYPE_REPEAT }));
        repeat_mapping = (KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    }
    [down_mapping, up_mapping, repeat_mapping]
}

pub fn map_click_to_action(from: &KeyClickActionWithMods, to: &KeyActionWithMods) -> [Mapping; 3] {
    let mut seq = vec![];

    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTCTRL.into(), value: TYPE_DOWN })); }
    if !from.modifiers.alt && to.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTALT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.right_alt && to.modifiers.right_alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_RIGHTALT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.shift && to.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTSHIFT.into(), value: TYPE_DOWN })); }
    if !from.modifiers.meta && to.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTMETA.into(), value: TYPE_DOWN })); }

    seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: to.key, value: to.value }));

    // revert to original
    if !from.modifiers.ctrl && to.modifiers.ctrl { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTCTRL.into(), value: TYPE_UP })); }
    if !from.modifiers.alt && to.modifiers.alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTALT.into(), value: TYPE_UP })); }
    if !from.modifiers.right_alt && to.modifiers.right_alt { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_RIGHTALT.into(), value: TYPE_UP })); }
    if !from.modifiers.shift && to.modifiers.shift { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTSHIFT.into(), value: TYPE_UP })); }
    if !from.modifiers.meta && to.modifiers.meta { seq.push(RuntimeKeyAction::KeyAction(KeyAction { key: KEY_LEFTMETA.into(), value: TYPE_UP })); }

    seq.push(RuntimeKeyAction::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

    let down_mapping = (KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() }, RuntimeAction::ActionSequence(seq));
    // stub up and repeat, click only triggers action on down press
    let up_mapping = (KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    let repeat_mapping = (KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers.clone() }, RuntimeAction::NOP);
    [down_mapping, up_mapping, repeat_mapping]
}


pub fn release_restore_modifiers(
    actual_state: &KeyModifierState,
    from_flags: &KeyModifierFlags,
    to_flags: &KeyModifierFlags,
    to_type: &i32,
) -> Vec<evdev_rs::InputEvent> {
    let mut output_events = vec![];

    // takes into account the actual state of a modifier and decides whether to release/restore it or not
    let mut release_or_restore_modifier = |is_actual_down: &bool, key: &Key| {
        if *to_type == 1 { // restore mods if actual mod is still pressed
            if *is_actual_down {
                output_events.push(
                    KeyAction { key: *key, value: *to_type }.to_input_ev()
                );
            }
        } else { // release mods if actual mod is still pressed (prob. always true since it was necessary to trigger the mapping)
            if *is_actual_down {
                output_events.push(
                    KeyAction { key: *key, value: *to_type }.to_input_ev()
                );
            }
        }
    };

    if from_flags.ctrl && !to_flags.ctrl {
        release_or_restore_modifier(&actual_state.left_ctrl, &KEY_LEFTCTRL.into());
        release_or_restore_modifier(&actual_state.right_ctrl, &KEY_RIGHTCTRL.into());
    }
    if from_flags.shift && !to_flags.shift {
        release_or_restore_modifier(&actual_state.left_shift, &KEY_LEFTSHIFT.into());
        release_or_restore_modifier(&actual_state.right_shift, &KEY_RIGHTSHIFT.into());
    }
    if from_flags.alt && !to_flags.alt {
        release_or_restore_modifier(&actual_state.left_alt, &KEY_LEFTALT.into());
    }
    if from_flags.right_alt && !to_flags.right_alt {
        release_or_restore_modifier(&actual_state.right_alt, &KEY_RIGHTALT.into());
    }
    if from_flags.meta && !to_flags.meta {
        release_or_restore_modifier(&actual_state.left_meta, &KEY_LEFTMETA.into());
        release_or_restore_modifier(&actual_state.right_meta, &KEY_RIGHTMETA.into());
    }

    if output_events.len() > 0 {
        output_events.push(SYN_REPORT.clone());
    }

    output_events

    // TODO eat keys we just released, un-eat keys we just restored
}