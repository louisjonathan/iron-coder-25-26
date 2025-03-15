// keyboard shortcut support

use std::collections::HashMap;
use std::fs;
use serde::{Deserialize};
use eframe::egui::Key;

#[derive(Deserialize)]
pub struct Keybinding {
    id: String,
    key: String,
}

impl Keybinding {
    fn to_key(&self) -> Option<Key> {
        Key::from_name(&self.key)
    }
}

pub struct Keybindings {
    bindings: HashMap<String, Key>,
}

impl Keybindings {
    pub fn new() -> Self {
        let file_content = fs::read_to_string("keybindings.json").expect("Unable to read file");
        let bindings: Vec<Keybinding> = serde_json::from_str(&file_content).expect("JSON was not well-formatted");
        let mut map = HashMap::new();
        for binding in bindings {
            if let Some(key) = binding.to_key() {
                map.insert(binding.id, key);
            }
        }
        Keybindings { bindings: map }
    }

    pub fn is_pressed(&self, ctx: &egui::Context, id: &str) -> bool {
        if let Some(key) = self.bindings.get(id) {
            ctx.input(|i| i.key_pressed(*key))
        } else {
            false
        }
    }
}
