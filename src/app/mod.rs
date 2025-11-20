#![allow(warnings)]
mod colorschemes;
pub mod icons;
mod ide_settings;
mod keybinding;
mod shared_state;
mod syntax_highlighting;
pub use shared_state::SharedState;

pub mod tabs;
pub use tabs::*;

pub mod window;
pub use window::MainWindow;

pub mod canvas_board;
pub use canvas_board::CanvasBoard;

pub mod canvas_connection;
pub use canvas_connection::CanvasConnection;

pub mod connection_wizard;

pub mod canvas_protocol;
pub use canvas_protocol::CanvasProtocol;

pub mod command;
pub use command::{Command, CommandHistory, AddProtocolConnectionCommand, RemoveProtocolConnectionCommand};

mod canvas_element;
