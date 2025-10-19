// keyboard shortcut support

use eframe::egui::Key;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
pub struct Keybinding {
    id: String,
    key: String,
    ctrl: bool,
    alt: bool,
}

impl Keybinding {
    fn to_key(&self) -> Option<Key> {
        Key::from_name(&self.key)
    }
}

pub struct Keybindings {
    bindings: HashMap<String, Keybinding>,
}

impl Keybindings {
    pub fn new() -> Self {
        let file_content =
            fs::read_to_string("resources/keybindings.json").expect("Unable to read file");
        let bindings: Vec<Keybinding> =
            serde_json::from_str(&file_content).expect("JSON was not well-formatted");
        let mut map = HashMap::new();
        for binding in bindings {
            map.insert(binding.id.clone(), binding);
        }
        Keybindings { bindings: map }
    }

    pub fn is_pressed(&self, ctx: &egui::Context, id: &str) -> bool {
        if let Some(binding) = self.bindings.get(id) {
            if let Some(key) = binding.to_key() {
                ctx.input(|i| {
                    let key_pressed = i.key_pressed(key);
                    let ctrl_pressed = binding.ctrl && i.modifiers.ctrl;
                    let alt_pressed = binding.alt && i.modifiers.alt;

                    key_pressed && (!binding.ctrl || ctrl_pressed) && (!binding.alt || alt_pressed)
                })
            } else {
                false
            }
        } else {
            false
        }
    }
}
