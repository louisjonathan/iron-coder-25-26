use crate::app::tabs::base_tab::BaseTab;
use crate::app::SharedState;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::{read_dir};

pub struct FileExplorerTab {
    root_dir: PathBuf,
    expanded_dirs: HashMap<PathBuf, Vec<PathBuf>>,
}

impl FileExplorerTab {
    pub fn new() -> Self {
        let root_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            root_dir,
            expanded_dirs: HashMap::new(),
        }
    }

    pub fn set_root_dir(&mut self, new_root: PathBuf) {
        self.root_dir = new_root;
        self.expanded_dirs.clear();
    }

    fn read_dir(dir: &PathBuf) -> Vec<PathBuf> {
        read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .collect()
            })
            .unwrap_or_else(|_| vec![])
    }

    fn toggle_dir(&mut self, dir: PathBuf) {
        if self.expanded_dirs.contains_key(&dir) {
            self.expanded_dirs.remove(&dir);
        } else {
            let contents = Self::read_dir(&dir);
            self.expanded_dirs.insert(dir, contents);
        }
    }
}

impl BaseTab for FileExplorerTab {
    fn draw(&mut self, ui: &mut egui::Ui, _state: &mut SharedState) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            fn draw_directory(
                ui: &mut egui::Ui,
                dir: &PathBuf,
                expanded_dirs: &HashMap<PathBuf, Vec<PathBuf>>,
                toggle_dir: &mut dyn FnMut(PathBuf),
                file_clicked: &mut dyn FnMut(PathBuf),
                max_visible: usize,
                visible_count: &mut usize,
                depth: usize,
            ) {
                if *visible_count >= max_visible {
                    return; // hack to avoid lag from having too many items open
                }

                let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
                ui.horizontal(|ui| {
                    ui.add_space(depth as f32 * 16.0);
                    if ui.button(format!("{}", dir_name)).clicked() {
                        toggle_dir(dir.clone());
                    }
                });

                *visible_count += 1;

                if let Some(contents) = expanded_dirs.get(dir) {
                    for entry in contents {
                        if *visible_count >= max_visible {
                            break;
                        }

                        if entry.is_dir() {
                            draw_directory(
                                ui,
                                entry,
                                expanded_dirs,
                                toggle_dir,
                                file_clicked,
                                max_visible,
                                visible_count,
                                depth + 1,
                            );
                        } else {
                            let file_name = entry.file_name().unwrap_or_default().to_string_lossy();
                            
                            // check if this is a supported file type
                            let is_supported_file = entry.extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| matches!(ext, "rs" | "json" | "txt"))
                                .unwrap_or(false);
                            
                            ui.horizontal(|ui| {
                                ui.add_space((depth + 1) as f32 * 16.0);
                                
                                if is_supported_file {
                                    // Make supported files clickable
                                    if ui.button(format!("{}", file_name)).clicked() {
                                        file_clicked(entry.clone());
                                    }
                                } else {
                                    // Not clickable
                                    ui.label(format!("{}", file_name));
                                }
                            });
                            *visible_count += 1;
                        }
                    }
                }
            }

            let expanded_dirs = self.expanded_dirs.clone();
            let root_dir = self.root_dir.clone();
            let mut toggle_dir = {
                let expanded_dirs = &mut self.expanded_dirs;
                move |dir: PathBuf| {
                    if expanded_dirs.contains_key(&dir) {
                        expanded_dirs.remove(&dir);
                    } else {
                        let contents = Self::read_dir(&dir);
                        expanded_dirs.insert(dir, contents);
                    }
                }
            };

            let mut file_clicked = |file_path: PathBuf| {
                _state.requested_file_to_open = Some(file_path);
            };

            let max_visible = 500;
            let mut visible_count = 0;
            draw_directory(
                ui,
                &root_dir,
                &expanded_dirs,
                &mut toggle_dir,
                &mut file_clicked,
                max_visible,
                &mut visible_count,
                0,
            );
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}