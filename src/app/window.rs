use crate::app::SharedState;
use crate::board::Board;
use super::tabs::*;

use crate::app::colorschemes::colorschemes;
use eframe::egui::{Ui};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::collections::HashMap;
use std::path::Path;

static OPENABLE_TABS: &'static [&'static str] = &[
    "Settings",
    "Canvas",
    "File Explorer",
    "Terminal",
    "Board Info",
	"Debug"
];

struct WindowContext<'a> {
    tabs: &'a mut HashMap<String, Box<dyn BaseTab>>,
    state: &'a mut SharedState,
}

impl<'a> egui_dock::TabViewer for WindowContext<'a> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        if let Some(tab) = self.tabs.get_mut(tab) {
            tab.draw(ui, self.state);
        }
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        self.tabs.remove(_tab);
        true
    }

    fn closeable(&mut self, _tab: &mut String) -> bool {
        if _tab == "Canvas" || _tab == "File Explorer" {
            false
        } else {
            true
        }
    }
}

pub struct MainWindow {
    tree: DockState<String>,
    tabs: HashMap<String, Box<dyn BaseTab>>,
    state: SharedState,
}

impl Default for MainWindow {
    fn default() -> Self {
        let mut tree = DockState::new(vec![
            "Canvas".to_owned(),
            "Settings".to_owned(),
            "./src/main.rs".to_owned(),
        ]);

        let [a, b] = tree.main_surface_mut().split_left(
            NodeIndex::root(),
            0.3,
            vec!["Board Info".to_owned()],
        );

        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.7, vec!["Terminal".to_owned()]);

        let mut tabs: HashMap<String, Box<dyn BaseTab>> = HashMap::new();

        tabs.insert("Board Info".to_string(), Box::new(BoardInfoTab::new()));
        tabs.insert("Canvas".to_string(), Box::new(CanvasTab::new()));
        tabs.insert("Settings".to_string(), Box::new(SettingsTab::new()));
        tabs.insert(
            "File Explorer".to_string(),
            Box::new(FileExplorerTab::new()),
        );
        tabs.insert("Terminal".to_string(), Box::new(TerminalTab::new()));

        let mut filetab = FileTab::default();
        let path = Path::new("./src/main.rs");
        filetab.load_from_file(path);
        tabs.insert(path.to_string_lossy().into_owned(), Box::new(filetab));

        Self {
            tree: tree,
            tabs: tabs,
            state: SharedState::default(),
        }
    }
}

impl MainWindow {
    pub fn display_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if self.tabs.contains_key("Settings") {
                //make sure settings tab gets current context
                //need help with this line
                let settings_tab = self
                    .tabs
                    .get_mut("Settings")
                    .unwrap()
                    .as_any_mut()
                    .downcast_mut::<SettingsTab>()
                    .unwrap();
                if settings_tab.should_random_colorscheme == true {
                    let random_choice = &colorschemes::get_random_color_scheme();
                    let colors = colorschemes::get_color_scheme(
                        &mut self.state.colorschemes,
                        random_choice,
                    );
                    self.state
                        .colorschemes
                        .set_color_scheme(&ctx, random_choice);

                    ui.visuals_mut().widgets.noninteractive.fg_stroke.color =
                        colors["extreme_bg_color"];
                    ui.visuals_mut().widgets.active.fg_stroke.color = colors["extreme_bg_color"];
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = colors["extreme_bg_color"];
                    ui.visuals_mut().widgets.open.fg_stroke.color = colors["extreme_bg_color"];

                    settings_tab.should_random_colorscheme = false;
                } else if settings_tab.should_example_colorscheme == true {
                    self.state.colorschemes.set_color_scheme(&ctx, &100);
                    settings_tab.should_example_colorscheme = false;
                }
            }
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Settings").clicked() {
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    for tab_name in OPENABLE_TABS {
                        let is_open = self.tabs.contains_key(*tab_name);
                        if ui
                            .add(egui::SelectableLabel::new(is_open, *tab_name))
                            .clicked()
                        {
                            if !is_open {
                                self.add_tab(tab_name.to_string());
                            }
                        }
                    }
                });
            });
        });
    }

    pub fn add_tab(&mut self, tab_name: String) {
        match tab_name.as_str() {
            "Settings" => {
                self.tabs
                    .insert(tab_name.clone(), Box::new(SettingsTab::new()));
            }
            "Canvas" => {
                self.tabs
                    .insert(tab_name.clone(), Box::new(CanvasTab::new()));
            }
            "Terminal" => {
                self.tabs
                    .insert(tab_name.clone(), Box::new(TerminalTab::new()));
            }
            "File Explorer" => {
                self.tabs
                    .insert(tab_name.clone(), Box::new(FileExplorerTab::new()));
            }
            "Board Info" => {
                self.tabs.insert(
                    tab_name.clone(),
                    Box::new(BoardInfoTab::new()),
                );
            }
			"Debug" => {
				self.tabs
                    .insert(tab_name.clone(), Box::new(DebugTab{}));
			}
            _ => {}
        }
        self.tree.push_to_focused_leaf(tab_name);
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.display_menu(ctx, _frame);

        if self.state.keybindings.is_pressed(ctx, "close_tab") {
            // close tab keybind for Jon... (once I figure out how to reliably find the current tab)
            println!("Close tab bind pressed...");
        }

        let mut context = WindowContext {
            tabs: &mut self.tabs,
            state: &mut self.state,
        };

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_leaf_collapse_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show(ctx, &mut context);
    }
}