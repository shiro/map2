use nom::Slice;

use crate::*;

#[derive(Default)]
pub struct SuffixTree<Value> {
    root: HashMap<char, SuffixTreeNode<Value>>,
}

impl<Value> SuffixTree<Value> {
    pub fn new() -> Self {
        Self {
            root: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: Value) {
        self.root
            .entry(key.chars().last().unwrap())
            .or_default()
            .insert(&key[0..key.len() - 1], value);
    }

    pub fn get(&self, key: &String) -> Option<&Value> {
        self.root
            .get(&key.chars().last().unwrap())
            .and_then(|x| x.get(&key[0..key.len() - 1]))
    }
}

impl<Value: Clone> Clone for SuffixTree<Value> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
        }
    }
}

pub struct SuffixTreeNode<Value> {
    value: Option<Value>,
    children: HashMap<char, SuffixTreeNode<Value>>,
}

impl<Value> Default for SuffixTreeNode<Value> {
    fn default() -> Self {
        Self {
            value: None,
            children: HashMap::new(),
        }
    }
}

impl<Value> SuffixTreeNode<Value> {
    pub fn new() -> Self {
        Self {
            value: None,
            children: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: Value) {
        if let Some(ch) = key.chars().last() {
            self.children
                .entry(ch)
                .or_default()
                .insert(key.slice(0..key.len() - 1), value);
        } else {
            self.value = Some(value);
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Some(ch) = key.chars().last() {
            self.children
                .get(&ch)
                .and_then(|x| x.get(key.slice(0..key.len() - 1)))
        } else {
            self.value.as_ref()
        }
    }
}

impl<Value: Clone> Clone for SuffixTreeNode<Value> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            children: self.children.clone(),
        }
    }
}
