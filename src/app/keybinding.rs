// keyboard shortcut support

use eframe::egui::Key;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Serialize, Clone)]
pub struct Keybinding {
    pub id: String,
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub description: String,
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

    pub fn get_all_keybindings(&self) -> Vec<&Keybinding> {
        self.bindings.values().collect()
    }

    pub fn get_all_keybindings_mut(&mut self) -> Vec<&mut Keybinding> {
        self.bindings.values_mut().collect()
    }

    pub fn get_keybinding(&self, id: &str) -> Option<&Keybinding> {
        self.bindings.get(id)
    }

    pub fn get_keybinding_mut(&mut self, id: &str) -> Option<&mut Keybinding> {
        self.bindings.get_mut(id)
    }

    pub fn add_keybinding(&mut self, keybinding: Keybinding) {
        self.bindings.insert(keybinding.id.clone(), keybinding);
    }

    pub fn remove_keybinding(&mut self, id: &str) -> Option<Keybinding> {
        self.bindings.remove(id)
    }

    pub fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bindings: Vec<&Keybinding> = self.bindings.values().collect();
        bindings.sort_by(|a, b| a.id.cmp(&b.id));
        
        let json = serde_json::to_string_pretty(&bindings)?;
        fs::write("resources/keybindings.json", json)?;
        Ok(())
    }

    pub fn reload_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file_content = fs::read_to_string("resources/keybindings.json")?;
        let bindings: Vec<Keybinding> = serde_json::from_str(&file_content)?;
        
        self.bindings.clear();
        for binding in bindings {
            self.bindings.insert(binding.id.clone(), binding);
        }
        Ok(())
    }
}
