use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Store {
    map: HashMap<String, String>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) -> Option<String> {
        self.map.insert(key, value)
    }

    pub fn get(&self, key: String) -> Option<String> {
        match self.map.get(&key) {
            Some(value) => Some(value.to_owned()),
            None => None,
        }
    }
}
