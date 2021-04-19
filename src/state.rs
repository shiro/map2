use crate::*;
use core::fmt::Alignment::Left;

#[derive(Eq, PartialEq, Hash)]
pub(crate) enum ModifierName {
    Tab,
    CapsLock,
    LeftCtrl,
    RightCtrl,
    LeftShift,
    RightShift,
    LeftMeta,
    RightMeta,
    LeftAlt,
    RightAlt,
}

pub(crate) enum SimpleModifierName {
    Tab,
    CapsLock,
    Ctrl,
    Shift,
    Meta,
    Alt,
}


#[derive(Clone)]
pub struct CompiledKeyMappings(pub(crate) HashMap<KeyActionWithMods, Arc<tokio::sync::Mutex<Block>>>);

impl CompiledKeyMappings { pub fn new() -> Self { CompiledKeyMappings(Default::default()) } }

pub struct State {
    pub(crate) modifiers: HashMap<ModifierName, bool>,

    pub(crate) tab_is_down: bool,
    pub(crate) capslock_is_down: bool,
    pub(crate) leftcontrol_is_down: bool,
    pub(crate) shift_is_down: bool,
    pub(crate) meta_is_down: bool,
    pub(crate) leftalt_is_down: bool,
    pub(crate) right_alt_is_down: bool,

    pub(crate) disable_alt_mod: bool,

    pub(crate) ignore_list: IgnoreList,
    // pub(crate) mappings: CompiledKeyMappings,

    pub(crate) active_window: Option<ActiveWindowResult>,
}


impl State {
    pub(crate) fn new() -> Self {
        let mut modifiers = HashMap::new();
        modifiers.insert(ModifierName::Tab, false);
        modifiers.insert(ModifierName::CapsLock, false);
        modifiers.insert(ModifierName::LeftCtrl, false);
        modifiers.insert(ModifierName::RightCtrl, false);
        modifiers.insert(ModifierName::LeftShift, false);
        modifiers.insert(ModifierName::RightShift, false);
        modifiers.insert(ModifierName::LeftMeta, false);
        modifiers.insert(ModifierName::RightMeta, false);
        modifiers.insert(ModifierName::LeftAlt, false);
        modifiers.insert(ModifierName::RightAlt, false);

        State {
            modifiers,
            tab_is_down: false,
            capslock_is_down: false,
            leftcontrol_is_down: false,
            shift_is_down: false,
            meta_is_down: false,
            leftalt_is_down: false,
            right_alt_is_down: false,
            disable_alt_mod: false,
            ignore_list: IgnoreList::new(),
            // mappings: CompiledKeyMappings::new(),
            active_window: None,
        }
    }

    pub(crate) fn get_modifier_state(&mut self, name: &ModifierName) -> &mut bool {
        self.modifiers.get_mut(name).unwrap()
    }

    // pub(crate) fn is_modifier_down_simple(&self, name: &SimpleModifierName) -> bool {
    //     use ModifierName::*;
    //     match name {
    //         SimpleModifierName::Tab => self.get_modifier_state(&Tab),
    //         SimpleModifierName::CapsLock => self.get_modifier_state(&CapsLock),
    //         SimpleModifierName::Ctrl => self.get_modifier_state(&LeftCtrl) || self.get_modifier_state(&RightCtrl),
    //         SimpleModifierName::Shift => self.get_modifier_state(&LeftShift) || self.get_modifier_state(&RightShift),
    //         SimpleModifierName::Meta => self.get_modifier_state(&LeftMeta) || self.get_modifier_state(&RightMeta),
    //         SimpleModifierName::Alt => self.get_modifier_state(&LeftAlt) || self.get_modifier_state(&RightAlt),
    //     }
    // }

    pub(crate) fn is_any_modifier_down(&self) -> bool {
        return self.leftalt_is_down || self.leftcontrol_is_down || self.shift_is_down || self.meta_is_down;
    }
}