use std::fmt::format;
use which::which;

use crate::app::colorschemes;
use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;
use egui_dropdown::DropDownBox;
use rfd::FileDialog;
pub struct SettingsTab {
    pub current_syntax_search: String,
    pub current_colorscheme_search: String,
    pub colorscheme_load_error_value: bool,
    pub syntax_theme_load_error_value: bool,
}
impl SettingsTab {
    pub fn new() -> Self {
        SettingsTab {
            current_colorscheme_search: String::new(),
            current_syntax_search: String::new(),
            colorscheme_load_error_value: false,
            syntax_theme_load_error_value: false,
        }
    }
}


impl BaseTab for SettingsTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.heading("Settings");
        ui.separator();
        ui.heading("Colorschemes");
        ui.horizontal(|ui|{
            ui.label("Current:");
            let colorscheme_filenames = &state.colorschemes.all_names;
            ui.add(
                DropDownBox::from_iter(
                    colorscheme_filenames,
                    "Select colorscheme",
                    &mut self.current_colorscheme_search,
                    |ui, text| ui.selectable_label(false, text),
                )
            );
            if ui.button("Save").clicked(){
                colorschemes::create_or_modify_colorscheme(&self.current_colorscheme_search, &state.colorschemes.current);
                self.colorscheme_load_error_value=false;
            }
            if ui.button("Load").clicked(){
                if(!state.colorschemes.try_use_colorscheme(ui, &self.current_colorscheme_search)){
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
	ui.collapsing("Terminal", |ui| {
		let candidates = ["bash", "cmd", "powershell", "sh"];
		let available_terms: Vec<_> = candidates
			.iter()
			.filter_map(|t| which::which(t).ok())
			.collect();

		for path in &available_terms {
			if ui.button(format!("Use {}", path.display())).clicked() {
				state.default_terminal = Some(path.to_path_buf());
			}
		}

		if ui.button("Choose Terminal Executable").clicked() {
			if let Some(picked_path) = rfd::FileDialog::new()
				.set_title("Select your preferred terminal")
				.pick_file()
			{
				state.default_terminal = Some(picked_path); // Use the PathBuf directly
			}
		}
	});
	ui.separator();
        ui.horizontal(|ui| {
            ui.label("Syntax Highlighting Theme:");
            let theme_names = state.syntax_highlighter.available_themes().clone();
            
            ui.add(
                DropDownBox::from_iter(
                    theme_names,
                    "Select theme",
                    &mut self.current_syntax_search,
                    |ui, text| ui.selectable_label(false, text),
                )
            );
            // if self.current_syntax_search != state.syntax_highlighter.current_theme {
            //     state.syntax_highlighter.set_theme(&self.current_syntax_search);
            // }
            if ui.button("Load").clicked(){
                if(!state.syntax_highlighter.set_theme(&self.current_syntax_search)){
                    self.syntax_theme_load_error_value=true;        
                }else{
                    self.syntax_theme_load_error_value=false;        
                };
            };
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
