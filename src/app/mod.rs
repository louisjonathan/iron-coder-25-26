#![allow(warnings)]
pub mod icons;
mod colorschemes;
mod keybinding;
mod syntax_highlighting;

mod shared_state;
use shared_state::{Message, SharedState};

pub mod tabs;
pub use tabs::*;

pub mod window;
pub use window::MainWindow;

mod canvas_board;
use canvas_board::CanvasBoard;

mod canvas_connection;
use canvas_connection::CanvasConnection;

mod canvas_element;