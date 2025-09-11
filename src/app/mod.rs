#![allow(warnings)]
pub mod icons;
mod colorschemes;
mod keybinding;

mod shared_state;
use shared_state::SharedState;

pub mod tabs;
pub use tabs::*;

pub mod window;
pub use window::MainWindow;
