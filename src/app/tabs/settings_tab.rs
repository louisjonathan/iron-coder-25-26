use crate::app::colorschemes;
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
        ui.heading("Colorschemes");
        ui.heading(format!("Current Colorscheme: {}", state.colorschemes.name));
        ui.collapsing("Modify current colorscheme", |ui| {
            let mut any_changed = false;

            let mut color_edit = |key: &str| {
                ui.horizontal(|ui| {
                    ui.label(key);
                    let response = ui
                        .color_edit_button_srgba(state.colorschemes.current.get_mut(key).unwrap());
                    if response.changed() {
                        any_changed = true;
                    }
                });
            };

            color_edit("extreme_bg_color");
            color_edit("faint_bg_color");
            color_edit("panel_fill");
            color_edit("window_fill");
            color_edit("window_stroke_color");
            color_edit("hyperlink_color");
            color_edit("error_fg_color");

            if any_changed {
                colorschemes::set_colorscheme(ui,&state.colorschemes.current);
            }
            ui.horizontal(|ui|{
                if ui.button("Save").clicked(){
                colorschemes::create_or_modify_colorscheme(&state.colorschemes.name, &state.colorschemes.current);
                }
                if ui.button("Cancel").clicked(){
                    state.colorschemes.try_use_colorscheme(ui, &state.colorschemes.name.clone());
                }
            });            
        });
        if ui.button("Create new colorscheme").clicked() {
            // popup or dropdown menu with:
            // - 5 color options, each an egui::widgets::color_picker_button
            // - Name entry box (the colorscheme will be saved to a toml file of this name)
            // - Save button
            // - Cancel button
        }
        if ui.button("Set example colorscheme").clicked() {
            state
                .colorschemes
                .try_use_colorscheme(ui, &"example_colorscheme.toml".to_string());
        }
        if ui.button("Set random colorscheme").clicked() {
            state.colorschemes.use_random_colorscheme(ui);
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
