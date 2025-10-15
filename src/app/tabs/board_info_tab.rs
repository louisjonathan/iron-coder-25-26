use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

use crate::app::CanvasBoard;

use crate::board;
use crate::board::display;
use serde::{Deserialize, Serialize};

use std::rc::Rc;
use std::cell::RefCell;

pub struct BoardInfoTab {
    chosen_board_idx: Option<usize>,
}

impl BoardInfoTab {
    pub fn new() -> Self {
        BoardInfoTab {
            chosen_board_idx: None,
        }
    }
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
            // if ui.button("Generate New Board").clicked() {
            //     todo!();
            // }
            ui.label("or select a board from the list below");
            ui.columns(num_cols, |columns| {
                for (i, b) in state.known_boards.clone().into_iter().enumerate() {
					if state.project.has_main_board() {
						let board = b.borrow();
						if board.is_main_board() {
							continue;
						}
					}
                    let col = i % num_cols;
                    // When a board is clicked, add it to the new project
                    ///@TODO  BoardSelectorWidget
                    if columns[col]
                        .add(board::display::BoardSelectorWidget(b.borrow().clone()))
                        .clicked()
                    {
                        state.project.add_board(&b);

						// state.project.add_board(b.clone());

                        // let board_rc = Rc::new(RefCell::new(CanvasBoard::new(&b).unwrap()));
                        // state.boards_used.push(board_rc);
                    }
                }

                // let last_col = state.boards_used.len();
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
