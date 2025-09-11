pub mod base_tab;

pub mod debug_tab;
pub mod canvas_tab;
pub mod file_tab;
pub mod file_explorer_tab;
pub mod terminal_tab;
pub mod board_info_tab;
pub mod settings_tab;

pub use debug_tab::DebugTab;
pub use canvas_tab::CanvasTab;
pub use file_tab::FileTab;
pub use file_explorer_tab::FileExplorerTab;
pub use terminal_tab::TerminalTab;
pub use board_info_tab::BoardInfoTab;
pub use settings_tab::SettingsTab;
pub use base_tab::BaseTab;