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
use std::process::{Child, Stdio};
use std::sync::mpsc::{self, Sender, Receiver};
use std::process::Command;
use std::io::BufRead;

use crate::board::Board;

pub struct SharedState {
    pub did_activate_colorscheme: bool,
    pub keybindings: Keybindings,
    pub colorschemes: colorscheme,
    pub syntax_highlighter: SyntaxHighlighter,
    pub project: Project,
    pub requested_file_to_open: Option<PathBuf>,
    pub known_boards: Vec<Rc<RefCell<Board>>>,
    pub terminal_buffer: String,
    pub tx: Option<Sender<String>>,
    pub rx: Option<Receiver<String>>,
    pub child: Option<Child>,
}

impl SharedState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn default() -> Self {
        use egui::epaint::color;

        let boards_dir = Path::new("./iron-coder-boards");
        let known_boards = board::get_boards(boards_dir);

        let mut project = Project::default();
        // match project.reload() {
        //     Ok(_) => (),
        //     Err(e) => println!("error reloading project from disk! {:?}", e),
        // }
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
        let terminal_buffer = last_settings.terminal_buffer.unwrap_or_default();

        Self {
            did_activate_colorscheme: false,
            keybindings: Keybindings::new(),
            colorschemes,
            syntax_highlighter: SyntaxHighlighter::new(),
            project,
            requested_file_to_open: None,
            known_boards,
            terminal_buffer,
            tx: None,
            rx: None,
            child: None,
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

    pub fn load_to_board(&mut self) {
        let (tx, rx) = mpsc::channel();
        if self.child.is_some() {
            let tx = tx.clone();
            tx.send("Use Ctrl+C to stop process before flashing again.".to_string()).unwrap();
            return;
        }
        if let Some(path) = &self.project.location {
            self.terminal_buffer.clear();
            self.tx = Some(tx.clone());
            self.rx = Some(rx);

            // Spawn cargo run
            let mut child = Command::new("cargo")
                .arg("run")
                .arg("--quiet")
                .current_dir(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            if let Some(stdout) = child.stdout.take() {
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let reader = std::io::BufReader::new(stdout).lines();
                    for line in reader {
                        let line = line.unwrap() + "\n";
                        tx.send(line).unwrap();
                    }
                });
            }
            if let Some(stderr) = child.stderr.take() {
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let reader = std::io::BufReader::new(stderr).lines();
                    for line in reader {
                        let line = line.unwrap() + "\n";
                        tx.send(line).unwrap();
                    }
                });
            }

            self.child = Some(child);
        }
    }

    pub fn stop_board(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            self.tx = None;
            self.rx = None;
            self.child = None;
            self.terminal_buffer.clear();
            self.terminal_buffer.push_str("\nProcess terminated\n");
        }
    }
    pub fn save_settings(&self) {
        let settings = IDE_Settings {
            colorscheme_file: Some(self.colorschemes.name.clone()),
            last_opened_project: self.project.location.clone(),
            opened_files: Vec::new(), // Future feature
            terminal_buffer: Some(self.terminal_buffer.clone()),
        };
        ide_settings::save_ide_settings(&settings);
    }
}