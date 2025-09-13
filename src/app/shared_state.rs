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
}