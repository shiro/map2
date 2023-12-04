use crate::*;

pub(crate) fn update_modifiers(modifiers: &mut Arc<KeyModifierState>, action: &KeyAction) -> bool {
    // TODO find a way to do this with a single accessor function
    let pairs: [(Key, fn(&KeyModifierState) -> bool, fn(&mut KeyModifierState) -> &mut bool); 8] = [
        (KEY_LEFTCTRL.into(), |s| s.left_ctrl, |s: &mut KeyModifierState| &mut s.left_ctrl),
        (KEY_RIGHTCTRL.into(), |s| s.right_ctrl, |s: &mut KeyModifierState| &mut s.right_ctrl),
        (KEY_LEFTALT.into(), |s| s.left_alt, |s: &mut KeyModifierState| &mut s.left_alt),
        (KEY_RIGHTALT.into(), |s| s.right_alt, |s: &mut KeyModifierState| &mut s.right_alt),
        (KEY_LEFTSHIFT.into(), |s| s.left_shift, |s: &mut KeyModifierState| &mut s.left_shift),
        (KEY_RIGHTSHIFT.into(), |s| s.right_shift, |s: &mut KeyModifierState| &mut s.right_shift),
        (KEY_LEFTMETA.into(), |s| s.left_meta, |s: &mut KeyModifierState| &mut s.left_meta),
        (KEY_RIGHTMETA.into(), |s| s.right_meta, |s: &mut KeyModifierState| &mut s.right_meta),
    ];

    for (key, is_modifier_down, modifier_mut) in pairs.iter() {
        if action.key.event_code == key.event_code && action.value == TYPE_DOWN && !is_modifier_down(&*modifiers) {
            let mut new_modifiers = modifiers.deref().deref().deref().clone();
            *modifier_mut(&mut new_modifiers) = true;
            *modifiers = Arc::new(new_modifiers);
            return true;
        } else if action.key.event_code == key.event_code && action.value == TYPE_UP {
            let mut new_modifiers = modifiers.deref().deref().deref().clone();
            *modifier_mut(&mut new_modifiers) = false;
            *modifiers = Arc::new(new_modifiers);
            return true;
            // TODO re-implement eating or throw it out completely
            // if ignore_list.is_ignored(&KeyAction::new(*key, TYPE_UP)) {
            //     ignore_list.unignore(&KeyAction::new(*key, TYPE_UP));
            //     return;
            // }
        } else if action.value == TYPE_REPEAT {
            return true;
        }
    };
    false
}