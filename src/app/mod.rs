#![allow(warnings)]
pub mod icons;
mod colorschemes;
mod keybinding;
mod syntax_highlighting;
mod ide_settings;
mod shared_state;
pub use shared_state::SharedState;

pub mod tabs;
pub use tabs::*;

pub mod window;
pub use window::MainWindow;

pub mod canvas_board;
pub use canvas_board::CanvasBoard;

pub mod canvas_connection;
pub use canvas_connection::CanvasConnection;

mod canvas_element;