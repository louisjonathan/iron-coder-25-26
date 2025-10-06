use crate::app::keybinding::{Keybinding, Keybindings};
use crate::board;
use crate::app::canvas_board::CanvasBoard;
use crate::project::Project;
use crate::app::colorschemes::{self, colorscheme};
use crate::app::ide_settings::{self, IDE_Settings};
use crate::app::syntax_highlighting::SyntaxHighlighter;
use std::path::{Path, PathBuf};
use crate::app::CanvasConnection;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use uuid::Uuid;
use std::io::BufRead;
use egui_term::{BackendCommand, TerminalBackend};
use std::ffi::OsStr;

use crate::board::Board;

pub struct SharedState {
    pub did_activate_colorscheme: bool,
    pub keybindings: Keybindings,
    pub colorschemes: colorscheme,
    pub syntax_highlighter: SyntaxHighlighter,
    pub project: Project,
    pub requested_file_to_open: Option<PathBuf>,
    pub known_boards: Vec<Rc<RefCell<Board>>>,
	pub default_terminal: Option<PathBuf>,
	pub output_terminal_backend: Option<Rc<RefCell<TerminalBackend>>>,
	pub reset_canvas: bool,
}

impl SharedState {
    #[cfg(not(target_arch = "wasm32"))]
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
            colorschemes= colorschemes::try_get_colorscheme(&scheme_name).map_or_else(
                || {
                    println!("Failed to load colorscheme {}, using default", scheme_name);
                    colorscheme::default()
                },
                |scheme| colorscheme {
                    current: scheme,
                    name: scheme_name.clone(),
                },
            );
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

		
        Self {
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
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        let boards: Vec<board::Board> = vec![board::Board::default()];

        #[cfg(target_arch = "wasm32")]
        let boards: Vec<board::Board> = vec![board::Board::default()];

        let mut project = Project::default();
        project.add_board(boards[0].clone());
        let boards_used = project.system.get_all_boards();
        Self {
            keybindings: Keybindings::new(),
            colorschemes: colorscheme::colorschemes::default(),
            syntax_highlighter: SyntaxHighlighter::new(),
            project: project,
            boards: boards,
            boards_used,
            requested_file_to_open: None,
        }
    }

pub fn term_open_project_dir(&mut self) {
	if let Some(term_ref) = &self.output_terminal_backend {
        let mut term = term_ref.borrow_mut();
        if let (Some(def_term), Some(dir)) = (&self.default_terminal, &self.project.location) {
			let term_type = def_term
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            let path_str = dir.to_string_lossy().replace("\\", "/");
            term.process_command(BackendCommand::Write(
				format!("cd {}\n", path_str).as_bytes().to_vec(),
			));
        }
    }
}

	pub fn build_project(&mut self) {
		if let Some(term_ref) = &self.output_terminal_backend {
			let mut term = term_ref.borrow_mut();
            if let Some(project_path) = &self.project.location {
                if Path::new(project_path).join("Makefile.toml").exists(){
                    term.process_command(BackendCommand::Write("cargo make modify-config-toml\n".as_bytes().to_vec()));
                }     
            }
			term.process_command(BackendCommand::Write("cargo +nightly build\n".as_bytes().to_vec()));
		}
	}
	
	pub fn run_project(&mut self) {
		if let Some(term_ref) = &self.output_terminal_backend {
			let mut term = term_ref.borrow_mut();
			term.process_command(BackendCommand::Write("cargo +nightly run\n".as_bytes().to_vec()));
		}
	}

	pub fn stop_board(&mut self) {
		if let Some(term_ref) = &self.output_terminal_backend {
			let mut term = term_ref.borrow_mut();
			term.process_command(BackendCommand::Write(vec![0x03]));
		}
	}
	
    pub fn save_settings(&self) {
        let settings = IDE_Settings {
            syntect_highlighting_file: Some(self.syntax_highlighter.get_current_theme().to_string()),
            colorscheme_file: Some(self.colorschemes.name.clone()),
            last_opened_project: self.project.location.clone(),
            opened_files: Vec::new(), // Future feature
			default_terminal: self.default_terminal.clone(),
        };
        ide_settings::save_ide_settings(&settings);
    }
}