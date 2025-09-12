use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

pub struct SettingsTab {
    pub should_random_colorscheme: bool,
    pub should_example_colorscheme: bool,
}
impl SettingsTab {
    pub fn new() -> Self {
        SettingsTab {
            should_random_colorscheme: false,
            should_example_colorscheme: false,
        }
    }
}

impl BaseTab for SettingsTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.heading("Settings");
        ui.label("Random setting 1");
        let mut s1_value = 50.0;
        ui.add(egui::Slider::new(&mut s1_value, 0.0..=100.0).text("Slider 1"));
        if ui.button("Set example colorscheme").clicked() {
            self.should_example_colorscheme = true;
        }
        if ui.button("Set random colorscheme").clicked() {
            self.should_random_colorscheme = true;
        }
        if ui.button("Save settings").clicked() {
            println!("Settings saved!");
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}