use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

use crate::app::CanvasBoard;

use crate::board;
use crate::board::display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct BoardInfoTab {
    chosen_board_idx: Option<usize>,
}

impl BoardInfoTab {
    pub fn new() -> Self {
        BoardInfoTab {
            chosen_board_idx: None,
        }
    }

    // /// Populate the project board list via the app-wide 'known boards' list
    // fn load_board_resources(&mut self) {
    //     info!("updating project boards from known boards list.");
    //     for b in state.project.system.get_all_boards_mut().iter_mut() {
    //         // returns true if the current, project board is equal to the current known_board
    //         let predicate = |known_board: &&Board| {
    //             return known_board == b;
    //         };
    //         if let Some(known_board) = self.known_boards.iter().find(predicate) {
    //             **b = known_board.clone();
    //         } else {
    //             warn!("Could not find the project board in the known boards list. Was the project manifest \
    //                    generated with an older version of Iron Coder?")
    //         }
    //     }
    // }
    // /// Display the list of available boards in a window, and return one if it was clicked
}

impl BaseTab for BoardInfoTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.heading("Board Selection");
        let mut board: Option<board::Board> = None;
        let available_width = ui.available_width();
        let mut num_cols = (available_width / 260.0) as usize;
        if num_cols == 0 {
            num_cols = 1;
        }
        egui::containers::scroll_area::ScrollArea::vertical().show(ui, |ui| {
            if ui.button("Generate New Board").clicked() {
                todo!();
            }
            ui.label("or select a board from the list below");
            ui.columns(num_cols, |columns| {
                for (i, b) in state.boards.clone().into_iter().enumerate() {
                    let col = i % num_cols;
                    // When a board is clicked, add it to the new project
                    ///@TODO  BoardSelectorWidget
                    if columns[col]
                        .add(board::display::BoardSelectorWidget(b.clone()))
                        .clicked()
                    {
						state.project.add_board(b.clone());
                        state.boards_used.push(CanvasBoard::new(&b).unwrap());
                        // board = Some(b.clone());
                        // self.chosen_board_idx = Some(i);
                    }
                }

                let last_col = state.boards_used.len();
            });
        });
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
