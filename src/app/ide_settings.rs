use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct IDE_Settings {
    pub colorscheme_file: Option<String>,
    pub last_opened_project: Option<PathBuf>,
    pub opened_files: Vec<PathBuf>,
    pub terminal_buffer: Option<String>,
}

pub fn save_ide_settings(settings: &IDE_Settings) {
    let target = PathBuf::from("./resources/ide_settings.toml");

    fs::write(&target, toml::to_string_pretty(settings).unwrap());
}
pub fn load_ide_settings() -> IDE_Settings {
    let target = PathBuf::from("./resources/ide_settings.toml");
    if target.is_file() {
        let file_content = fs::read_to_string(&target).expect("Unable to read file");
        toml::from_str(&file_content).expect("JSON was not well-formatted")
    } else {
        IDE_Settings::default()
    }
}

impl Default for IDE_Settings {
    fn default() -> Self {
        IDE_Settings {
            colorscheme_file: None,
            last_opened_project: None,
            opened_files: Vec::new(),
            terminal_buffer: None,
        }
    }
}   
