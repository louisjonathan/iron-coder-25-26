use crate::app::canvas_board::CanvasBoard;
use crate::app::colorschemes::{self, colorscheme};
use crate::app::command::CommandHistory;
use crate::app::connection_wizard::ConnectionWizard;
use crate::app::ide_settings::{self, IDE_Settings};
use crate::app::keybinding::{Keybinding, Keybindings};
use crate::app::syntax_highlighting::SyntaxHighlighter;
use crate::board::{self, Board};
use crate::project::Project;

use crate::app::CanvasConnection;
use eframe::glow::LINE;
use egui_term::{BackendCommand, TerminalBackend};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use uuid::Uuid;

#[cfg(target_os = "windows")]
pub const LINE_ENDING: &str = "\r\n";
#[cfg(not(target_os = "windows"))]
pub const LINE_ENDING: &str = "\n";

pub struct SharedState {
    pub did_activate_colorscheme: bool,
    pub keybindings: Keybindings,
    pub colorschemes: colorscheme,
    pub syntax_highlighter: SyntaxHighlighter,
    pub project: Project,
    pub requested_file_to_open: Option<PathBuf>,
    pub known_boards: Vec<Rc<Board>>,
    pub default_terminal: Option<PathBuf>,
    pub output_terminal_backend: Option<Rc<RefCell<TerminalBackend>>>,
    pub reset_canvas: bool,
    pub sync_file_explorer: bool,
    pub connection_wizard: Option<ConnectionWizard>,
    pub command_history: CommandHistory,
}

impl SharedState {
    pub fn default() -> Self {
        use egui::epaint::color;

        let boards_dir = Path::new("./iron-coder-boards");
        let known_boards = board::get_boards(boards_dir);

        let mut project = Project::default();
        let last_settings = ide_settings::load_ide_settings();
        if let Some(location) = last_settings.last_opened_project {
            project.location = Some(location.clone());
            match project.load_from(&location, &known_boards) {
                Ok(_) => (),
                Err(e) => println!("Error loading project! {:?}", e),
            }
        }
        let mut colorschemes = colorscheme::default();

        if let Some(scheme_name) = last_settings.colorscheme_file {
            match colorschemes::try_get_colorscheme(&scheme_name) {
                Some(scheme) => {
                    colorschemes.all_names = colorschemes::get_colorscheme_filenames();
                    colorschemes.current = scheme.clone();
                    colorschemes.name = scheme_name.clone();

                    // Recalculate contrast colors for the loaded colorscheme (using canvas background)
                    let background = scheme
                        .get("window_fill")
                        .copied()
                        .unwrap_or(egui::Color32::GRAY);
                    colorschemes.contrast_colors =
                        colorschemes::calculate_contrast_colors(&background, &scheme);
                }
                None => {
                    println!("Failed to load colorscheme {}, using default", scheme_name);
                    colorschemes = colorscheme::default();
                }
            }
        }

        let mut syntax_highlighter = SyntaxHighlighter::new();
        if let Some(theme_name) = last_settings.syntect_highlighting_file {
            syntax_highlighter.set_theme(&theme_name);
        }

        let default_terminal = if let Some(path) = last_settings.default_terminal {
            path
        } else if cfg!(target_os = "windows") {
            PathBuf::from("cmd")
        } else {
            PathBuf::from("bash")
        };

        let mut state = Self {
            did_activate_colorscheme: false,
            keybindings: Keybindings::new(),
            colorschemes,
            syntax_highlighter,
            project,
            requested_file_to_open: None,
            known_boards,
            output_terminal_backend: None,
            default_terminal: Some(default_terminal),
            reset_canvas: false,
            sync_file_explorer: false,
            connection_wizard: None,
            command_history: CommandHistory::new(),
        };

        // Update all wire colors to match the loaded colorscheme
        state.update_all_wire_colors_to_match_colorscheme();

        state
    }

    pub fn term_open_project_dir(&mut self) {
        if let Some(term_ref) = &self.output_terminal_backend {
            let mut term = term_ref.borrow_mut();
            if let (Some(def_term), Some(project_dir)) =
                (&self.default_terminal, &self.project.location)
            {
                let term_type = def_term
                    .file_name()
                    .and_then(OsStr::to_str)
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let path_str = project_dir.to_string_lossy().replace("\\", "/");
                term.process_command(BackendCommand::Write(
                    format!("cd {}{}", path_str, LINE_ENDING)
                        .as_bytes()
                        .to_vec(),
                ));
                // Clear the terminal after changing directory
                if term_type.contains("cmd") || term_type.contains("powershell") {
                    term.process_command(BackendCommand::Write(
                        format!("cls {}", LINE_ENDING).as_bytes().to_vec(),
                    ));
                } else {
                    term.process_command(BackendCommand::Write(
                        format!("clear {}", LINE_ENDING).as_bytes().to_vec(),
                    ));
                }
            }
        }
        self.sync_file_explorer = true;
    }

    pub fn clear_terminal(&mut self) {
        if let Some(term_ref) = &self.output_terminal_backend {
            let mut term = term_ref.borrow_mut();
            if let Some(def_term) = &self.default_terminal {
                let term_type = def_term
                    .file_name()
                    .and_then(OsStr::to_str)
                    .unwrap_or("")
                    .to_ascii_lowercase();
                term.process_command(BackendCommand::Write(vec![0x03]));
                if term_type.contains("cmd") || term_type.contains("powershell") {
                    term.process_command(BackendCommand::Write(
                        format!("cls {}", LINE_ENDING).as_bytes().to_vec(),
                    ));
                } else {
                    term.process_command(BackendCommand::Write(
                        format!("clear {}", LINE_ENDING).as_bytes().to_vec(),
                    ));
                }
            }
        }
    }

    pub fn build_project(&mut self) {
        if let Some(term_ref) = &self.output_terminal_backend {
            let mut term = term_ref.borrow_mut();
            self.project.update_toolchain_location();
            term.process_command(BackendCommand::Write(vec![0x03]));
            term.process_command(BackendCommand::Write(
                format!("cargo +nightly build{}", LINE_ENDING)
                    .as_bytes()
                    .to_vec(),
            ));
        }
    }

    pub fn run_project(&mut self) {
        if let Some(term_ref) = &self.output_terminal_backend {
            let mut term = term_ref.borrow_mut();
            // if project uses main board arduino, select --target atmega328p.json
            // if let Some(board) = self.project.main_board.as_ref() {
            //     if board.borrow().board.cpu.as_deref() == Some("arduino-uno") {
            //         term.process_command(BackendCommand::Write(
            //             format!("cargo +nightly run {}", LINE_ENDING).as_bytes().to_vec(),
            //         ));
            //     } else {
            //         term.process_command(BackendCommand::Write(
            //             format!("cargo +nightly run --target avr-atmega328p.json{}", LINE_ENDING).as_bytes().to_vec(),
            //         ));
            //     }
            // }
            term.process_command(BackendCommand::Write(vec![0x03]));
            term.process_command(BackendCommand::Write(
                format!("cargo +nightly run{}", LINE_ENDING)
                    .as_bytes()
                    .to_vec(),
            ));
        }
    }

    pub fn save_settings(&self) {
        let settings = IDE_Settings {
            syntect_highlighting_file: Some(
                self.syntax_highlighter.get_current_theme().to_string(),
            ),
            colorscheme_file: Some(self.colorschemes.name.clone()),
            last_opened_project: self.project.location.clone(),
            opened_files: Vec::new(), // Future feature
            default_terminal: self.default_terminal.clone(),
        };
        ide_settings::save_ide_settings(&settings);
    }
    pub fn get_ide_installation_path() -> PathBuf {
        if let Some(proj_dir) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        {
            if proj_dir.ends_with("debug") || proj_dir.ends_with("release") {
                if let Some(parent) = proj_dir.parent() {
                    return parent.to_path_buf();
                }
            } else {
                return proj_dir;
            }
        }
        PathBuf::from(".")
    }

    /// Update all wire colors to match the current colorscheme's secondary contrast color
    pub fn update_all_wire_colors_to_match_colorscheme(&mut self) {
        let wire_color = self
            .colorschemes
            .contrast_colors
            .get(1)
            .copied()
            .unwrap_or(egui::Color32::WHITE);

        for connection in self.project.connections_iter() {
            connection.borrow_mut().color = wire_color;
        }
    }
}
