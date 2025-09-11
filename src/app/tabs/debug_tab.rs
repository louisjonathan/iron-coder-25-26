use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

pub struct DebugTab {
}

impl DebugTab {
    pub fn default() -> Self {
        Self {
        }
    }
}

impl BaseTab for DebugTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
		ui.label("Testing!");
		
	}

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
