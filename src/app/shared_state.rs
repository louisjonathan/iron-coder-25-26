use crate::app::keybinding::{Keybinding, Keybindings};
use crate::board;
use crate::app::canvas_board::CanvasBoard;
use crate::project::Project;
use crate::app::colorschemes::colorschemes;
use crate::app::syntax_highlighting::SyntaxHighlighter;
use std::path::{Path, PathBuf};
use crate::app::CanvasConnection;
use std::rc::Rc;
use std::cell::RefCell;

pub struct SharedState {
    pub keybindings: Keybindings,
    pub colorschemes: colorschemes,
    pub syntax_highlighter: SyntaxHighlighter,
    pub project: Project,
    pub boards: Vec<board::Board>,
    pub boards_used: Vec<Rc<RefCell<CanvasBoard>>>,
    pub connections: Vec<Rc<RefCell<CanvasConnection>>>,
    pub requested_file_to_open: Option<PathBuf>,
}

impl SharedState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn default() -> Self {
        let boards_dir = Path::new("./iron-coder-boards");
        let boards: Vec<board::Board> = board::get_boards(boards_dir);

        let mut project = Project::default();
        project.known_boards = boards.clone();
        // match project.reload() {
        //     Ok(_) => (),
        //     Err(e) => println!("error reloading project from disk! {:?}", e),
        // }

        let boards_used = Vec::new();
        
        let mut connections = Vec::new();

        Self {
            keybindings: Keybindings::new(),
            colorschemes: colorschemes::default(),
            syntax_highlighter: SyntaxHighlighter::new(),
            project,
            boards,
            boards_used,
            connections,
            requested_file_to_open: None,
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
            colorschemes: colorschemes::colorschemes::default(),
            syntax_highlighter: SyntaxHighlighter::new(),
            project: project,
            boards: boards,
            boards_used,
            requested_file_to_open: None,
        }
    }

    pub fn add_board(&mut self, b: &board::Board) {

        match b.is_main_board() {
            true => {
                if self.project.has_main_board() {
                    // info!("project already contains a main board! aborting.");
                    self.project.terminal_buffer += "Project already contains a main board\n";
                    return;
                } else {
                    self.project.system.main_board = Some(b.clone());
                    let board_rc = Rc::new(RefCell::new(CanvasBoard::new(&b).unwrap()));
                    self.boards_used.push(board_rc);
                }
            }
            false => {
                // don't duplicate a board
                if self.project.system.peripheral_boards.contains(&b) {
                    // info!(
                    //     "project <{}> already contains board <{:?}>",
                    //     self.name, board
                    // );
                    self.project.terminal_buffer += "Project already contains that board\n";
                    return;
                } else {
                    self.project.system.peripheral_boards.push(b.clone());
                    let board_rc = Rc::new(RefCell::new(CanvasBoard::new(&b).unwrap()));
                    self.boards_used.push(board_rc);
                }
            }
        }
    }

    pub fn load_boards_from_project(&mut self) {
        if let Some(b) = &self.project.system.main_board {
            if let Some(cb) = CanvasBoard::new(&b) {
                self.boards_used.push(Rc::new(RefCell::new(cb)));
            }
        }

        for b in &self.project.system.peripheral_boards {
            if let Some(cb) = CanvasBoard::new(&b) {
                self.boards_used.push(Rc::new(RefCell::new(cb)));
            }
        }
    }
}