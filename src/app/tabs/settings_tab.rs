use std::fmt::format;

use crate::app::colorschemes;
use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;
pub struct SettingsTab {
    pub colorscheme_load_error_value: bool,
}
impl SettingsTab {
    pub fn new() -> Self {
        SettingsTab {
            colorscheme_load_error_value: false,
        }
    }
}


impl BaseTab for SettingsTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.heading("Settings");
        ui.separator();
        ui.heading("Colorschemes");
        ui.horizontal(|ui|{
            ui.label("Current Colorscheme:  ");
            ui.text_edit_singleline(&mut state.colorschemes.name);
            if ui.button("Save").clicked(){
                colorschemes::create_or_modify_colorscheme(&state.colorschemes.name, &state.colorschemes.current);
                self.colorscheme_load_error_value=false;
            }
            if ui.button("Load").clicked(){
                if(!state.colorschemes.try_use_colorscheme(ui, &state.colorschemes.name.clone())){
                    self.colorscheme_load_error_value=true;        
                }else{
                    self.colorscheme_load_error_value=false;        
                };
            };
            if self.colorscheme_load_error_value {
                ui.label("Error! File not found!!");
            };
            
        });
        ui.collapsing("Colors", |ui| {
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
            color_edit("code_bg_color");
            color_edit("panel_fill");
            color_edit("window_fill");

            if any_changed {
                colorschemes::set_colorscheme(ui,&state.colorschemes.current);
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Syntax Highlighting Theme:");
            let theme_names = state.syntax_highlighter.available_themes();
            let mut selected_theme = state.syntax_highlighter.get_current_theme().to_string();
            egui::ComboBox::from_id_source("syntax_theme_combo")
            .selected_text(&selected_theme)
            .show_ui(ui, |ui| {
                for theme in theme_names {
                ui.selectable_value(&mut selected_theme, theme.clone(),theme);
                }
            });
            if selected_theme != state.syntax_highlighter.get_current_theme() {
            state.syntax_highlighter.set_theme(&selected_theme);
            }
        });
        ui.separator();
        if ui.button("Set example colorscheme").clicked() {
            state
                .colorschemes
                .try_use_colorscheme(ui, &"example_colorscheme.toml".to_string());
        }
        if ui.button("Set random colorscheme").clicked() {
            state.colorschemes.use_random_colorscheme(ui);
        }
        if ui.button("Save settings").clicked() {
            state.save_settings();
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
