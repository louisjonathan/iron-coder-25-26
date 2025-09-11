use crate::app::keybinding::{Keybinding, Keybindings};
use crate::board;
use crate::project::Project;
use crate::app::colorschemes::colorschemes;
use std::path::Path;

pub struct SharedState {
    pub keybindings: Keybindings,
    pub colorschemes: colorschemes,
    pub project: Project,
    pub boards: Vec<board::Board>,
    pub boards_used: Vec<board::Board>,
}

impl SharedState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn default() -> Self {
        let boards_dir = Path::new("./iron-coder-boards");
        let boards: Vec<board::Board> = board::get_boards(boards_dir);

        let mut project = Project::default();
        let boards_used = project.system.get_all_boards();
        Self {
            keybindings: Keybindings::new(),
            colorschemes: colorschemes::default(),
            project: project,
            boards: boards,
            boards_used,
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
            project: project,
            boards: boards,
            boards_used,
        }
    }
}