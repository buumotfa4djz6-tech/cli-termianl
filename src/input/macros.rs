use std::collections::HashMap;

/// F-key macro bindings.
pub struct MacroManager {
    bindings: HashMap<String, String>,
}

impl MacroManager {
    pub fn new(bindings: HashMap<String, String>) -> Self {
        Self { bindings }
    }

    /// Resolve a key like "F2" to a command string.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.bindings.get(key).map(|s| s.as_str())
    }

    pub fn update(&mut self, bindings: HashMap<String, String>) {
        self.bindings = bindings;
    }
}
