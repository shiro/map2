use crate::*;

pub struct State {
    pub(crate) tab_is_down: bool,
    pub(crate) capslock_is_down: bool,
    pub(crate) leftcontrol_is_down: bool,
    pub(crate) shift_is_down: bool,
    pub(crate) meta_is_down: bool,
    pub(crate) leftalt_is_down: bool,
    pub(crate) right_alt_is_down: bool,

    pub(crate) disable_alt_mod: bool,

    pub(crate) ignore_list: IgnoreList,
    pub(crate) mappings: KeyMappings,

    pub(crate) active_window: Option<ActiveWindowResult>,
}

impl State {
    pub(crate) fn is_modifier_down(&self) -> bool {
        return self.leftalt_is_down || self.leftcontrol_is_down || self.shift_is_down || self.meta_is_down;
    }
}