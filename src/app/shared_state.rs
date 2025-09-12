use crate::app::keybinding::{Keybinding, Keybindings};
use crate::board;
use crate::app::canvas_board::CanvasBoard;
use crate::project::Project;
use crate::app::colorschemes::colorschemes;
use std::path::Path;
use crate::app::CanvasConnection;

pub struct SharedState {
    pub keybindings: Keybindings,
    pub colorschemes: colorschemes,
    pub project: Project,
    pub boards: Vec<board::Board>,
    pub boards_used: Vec<CanvasBoard>,
    pub connections: Vec<CanvasConnection>,
}

impl SharedState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn default() -> Self {
        let boards_dir = Path::new("./iron-coder-boards");
        let boards: Vec<board::Board> = board::get_boards(boards_dir);

        let mut project = Project::default();
        let boards_used = Vec::new();
        
        let mut connections = Vec::new();
        connections.push(CanvasConnection::new());

        Self {
            keybindings: Keybindings::new(),
            colorschemes: colorschemes::default(),
            project,
            boards,
            boards_used,
            connections,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn default() -> Self {
        use std::default;

        let boards: Vec<board::Board> = vec![board::Board::default()];
        
        let mut connections = Vec::new();
        connections.push(CanvasConnection::new());

        let mut project = Project::default();
        project.add_board(boards[0].clone());
        let boards_used = project.system.get_all_boards();
        Self {
            keybindings: Keybindings::new(),
            colorschemes: colorschemes::default(),
            project: project,
            boards: boards,
            boards_used: Vec::new(),
            connections,
        }
    }
}