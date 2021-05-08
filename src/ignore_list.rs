use crate::*;

pub struct IgnoreList(Vec<KeyAction>);

impl IgnoreList {
    pub fn new() -> Self { IgnoreList(Default::default()) }

    pub fn is_ignored(&self, key: &KeyAction) -> bool {
        match self.0.iter().position(|x| x == key) {
            None => false,
            Some(_) => true
        }
    }

    pub fn unignore(&mut self, key: &KeyAction) {
        if let Some(pos) = self.0.iter().position(|x| x == key) {
            self.0.remove(pos);
        }
    }

    pub fn ignore(&mut self, key: &KeyAction) {
        if let None = self.0.iter().position(|x| x == key) {
            self.0.push(*key);
        }
    }
}
