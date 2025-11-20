use std::fmt::format;
use which::which;

use crate::app::SharedState;
use crate::app::colorschemes;
use crate::app::keybinding::Keybinding;
use crate::app::tabs::base_tab::BaseTab;
use egui_dropdown::DropDownBox;
use rfd::FileDialog;

pub struct SettingsTab {
    pub current_syntax_search: String,
    pub current_colorscheme_search: String,
    pub colorscheme_load_error_value: bool,
    pub syntax_theme_load_error_value: bool,

    // Keybinding editor state
    pub keybinding_save_success: bool,
    pub editing_keybindings: Vec<Keybinding>, // Local copy for editing
    pub keybindings_loaded: bool,
}
impl SettingsTab {
    pub fn new() -> Self {
        SettingsTab {
            current_colorscheme_search: String::new(),
            current_syntax_search: String::new(),
            colorscheme_load_error_value: false,
            syntax_theme_load_error_value: false,

            keybinding_save_success: false,
            editing_keybindings: Vec::new(),
            keybindings_loaded: false,
        }
    }
}

impl BaseTab for SettingsTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.heading("Settings");
        ui.separator();
        ui.heading("Colorschemes");
        ui.horizontal(|ui| {
            ui.label("Current:");
            let colorscheme_filenames = &state.colorschemes.all_names;
            ui.add(DropDownBox::from_iter(
                colorscheme_filenames,
                "Select colorscheme",
                &mut self.current_colorscheme_search,
                |ui, text| ui.selectable_label(false, text),
            ));
            if ui.button("Save").clicked() {
                colorschemes::create_or_modify_colorscheme(
                    &self.current_colorscheme_search,
                    &state.colorschemes.current,
                );
                self.colorscheme_load_error_value = false;
            }
            if ui.button("Load").clicked() {
                if (!state
                    .colorschemes
                    .try_use_colorscheme(ui, &self.current_colorscheme_search))
                {
                    self.colorscheme_load_error_value = true;
                } else {
                    self.colorscheme_load_error_value = false;

                    // Update all existing wire colors to match new colorscheme
                    state.update_all_wire_colors_to_match_colorscheme();
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
                colorschemes::set_colorscheme(ui, &state.colorschemes.current);
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
                    state.default_terminal = Some(picked_path);
                }
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Syntax Highlighting Theme:");
            let theme_names = state.syntax_highlighter.available_themes().clone();

            ui.add(DropDownBox::from_iter(
                theme_names,
                "Select theme",
                &mut self.current_syntax_search,
                |ui, text| ui.selectable_label(false, text),
            ));
            if ui.button("Load").clicked() {
                if (!state
                    .syntax_highlighter
                    .set_theme(&self.current_syntax_search))
                {
                    self.syntax_theme_load_error_value = true;
                } else {
                    self.syntax_theme_load_error_value = false;
                };
            };
            if ui.button("Browse ...").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter(".tmTheme", &["tmTheme"])
                    .set_title("Select a .tmTheme file")
                    .pick_file()
                {
                    match std::fs::File::open(&path) {
                        Ok(file) => {
                            state.syntax_highlighter.try_add_file_else_update(file);
                            self.syntax_theme_load_error_value = false;
                            self.current_syntax_search = path
                                .file_stem()
                                .and_then(|stem| stem.to_str())
                                .unwrap_or_default()
                                .to_string();
                        }
                        Err(e) => {
                            self.syntax_theme_load_error_value = true;
                            eprintln!("Failed to open file: {}", e);
                        }
                    }
                }
            }
            if self.syntax_theme_load_error_value {
                ui.colored_label(egui::Color32::RED, "Error loading syntax theme!");
            }
        });

        ui.separator();
        if ui.button("Set example colorscheme").clicked() {
            state
                .colorschemes
                .try_use_colorscheme(ui, &"example_colorscheme.toml".to_string());
            self.current_colorscheme_search = "example_colorscheme.toml".to_string();
            state.update_all_wire_colors_to_match_colorscheme();
        }
        if ui.button("Set random colorscheme").clicked() {
            state.colorschemes.use_random_colorscheme(ui);
            self.current_colorscheme_search = state.colorschemes.name.clone();
            state.update_all_wire_colors_to_match_colorscheme();
        }
        if ui.button("Save settings").clicked() {
            state.save_settings();
        }

        ui.separator();
        ui.heading("Keybindings");

        egui::Frame::none()
            .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
            .inner_margin(10.0)
            .show(ui, |ui| {
                // Load keybindings if not already loaded
                if !self.keybindings_loaded {
                    self.editing_keybindings = state
                        .keybindings
                        .get_all_keybindings()
                        .iter()
                        .map(|kb| (*kb).clone())
                        .collect();
                    self.keybindings_loaded = true;
                }

                // Display save status messages
                if self.keybinding_save_success {
                    ui.colored_label(egui::Color32::GREEN, "Keybindings saved successfully!");
                }

                // Buttons for managing keybindings
                ui.horizontal(|ui| {
                    if ui.button("Apply Changes").clicked() {
                        self.apply_keybinding_changes(state);
                    }

                    if ui.button("Reset to Current").clicked() {
                        self.editing_keybindings = state
                            .keybindings
                            .get_all_keybindings()
                            .iter()
                            .map(|kb| (*kb).clone())
                            .collect();
                        self.keybinding_save_success = false;
                    }

                    if ui.button("Reset to Defaults").clicked() {
                        self.reset_to_default_keybindings(state);
                    }
                });

                ui.separator();

                // Keybinding list
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .min_scrolled_height(300.0)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for keybinding in self.editing_keybindings.iter_mut() {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("ID:");
                                        ui.label(&keybinding.id);
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Description:");
                                        ui.label(&keybinding.description);
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Key:");
                                        let mut key_input = keybinding.key.clone();
                                        if ui.text_edit_singleline(&mut key_input).changed() {
                                            keybinding.key = key_input.to_uppercase();
                                        }

                                        ui.checkbox(&mut keybinding.ctrl, "Ctrl");
                                        ui.checkbox(&mut keybinding.alt, "Alt");
                                    });

                                    // Display current combination
                                    let combo = format_keybinding_combo(keybinding);
                                    ui.label(format!("Combination: {}", combo));
                                });
                            });
                            ui.add_space(5.0);
                        }
                    });
            });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SettingsTab {
    fn apply_keybinding_changes(&mut self, state: &mut SharedState) {
        // Clear existing keybindings
        for keybinding in &self.editing_keybindings {
            // Remove old version if it exists
            state.keybindings.remove_keybinding(&keybinding.id);
            // Add the updated version
            state.keybindings.add_keybinding(keybinding.clone());
        }

        // Save to file
        if state.keybindings.save_to_file().is_ok() {
            self.keybinding_save_success = true;
            println!("Keybindings saved successfully");
        }
    }

    fn reset_to_default_keybindings(&mut self, state: &mut SharedState) {
        use std::fs;

        if fs::copy(
            "resources/keybindings_default.json",
            "resources/keybindings.json",
        )
        .is_ok()
        {
            if state.keybindings.reload_from_file().is_ok() {
                self.editing_keybindings = state
                    .keybindings
                    .get_all_keybindings()
                    .iter()
                    .map(|kb| (*kb).clone())
                    .collect();

                self.keybinding_save_success = true;
                println!("Keybindings reset to defaults successfully");
            }
        }
    }
}

fn format_keybinding_combo(keybinding: &Keybinding) -> String {
    let mut parts = Vec::new();

    if keybinding.ctrl {
        parts.push("Ctrl".to_string());
    }
    if keybinding.alt {
        parts.push("Alt".to_string());
    }
    parts.push(keybinding.key.to_uppercase());

    parts.join(" + ")
}
