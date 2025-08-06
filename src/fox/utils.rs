use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CodeLocation {
    line: usize,
    abs_position: usize,
}

impl CodeLocation {
    pub fn new(line: usize, abs_position: usize) -> Self {
        Self { line, abs_position }
    }

    pub fn line_number(&self) -> usize {
        self.line
    }

    pub fn absolute_position(&self) -> usize {
        self.abs_position
    }
}

impl Default for CodeLocation {
    fn default() -> Self {
        Self {
            line: 1,
            abs_position: 0,
        }
    }
}

/// convention function to create mutable pointer
///
pub fn mutable_cell<T>(value: T) -> std::rc::Rc<std::cell::RefCell<T>> {
    std::rc::Rc::new(std::cell::RefCell::new(value))
}

/// Fill hash for map of <Hashable1: Hashable2>
///
pub fn fill_hash<H, K, V>(map: &HashMap<K, V>, state: &mut H)
where
    H: std::hash::Hasher,
    K: Hash + Ord,
    V: Hash,
{
    let mut keys: Vec<_> = map.keys().collect();
    keys.sort();
    for key in keys {
        key.hash(state);
        map[key].hash(state);
    }
}
