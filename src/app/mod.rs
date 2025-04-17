pub mod colorschemes;
pub mod keybinding;
pub mod icons;

// This is the example from https://github.com/Adanos020/egui_dock/blob/main/examples/hello.rs
// Modified for the purposes of Iron Coder https://github.com/shulltronics/iron-coder

use std::str::FromStr;

use std::rc::Rc;
use std::cell::RefCell;
use crate::board::Board;

use std::path::Path;

use eframe::egui::{Pos2, Rect, Sense, Ui, Vec2};
use eframe::{egui, NativeOptions};
use egui::Area;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use emath::{self};

use crate::board;
use crate::project::Project;

use std::collections::HashMap;

use eframe::egui::Key;
use serde::{Deserialize, Serialize};

use crate::app::keybinding::{Keybinding, Keybindings};


static OPENABLE_TABS: &'static [&'static str] = &[
    "Settings",
    "Canvas",
    "File Explorer",
    "Terminal",
    "Board Info",
];

trait BaseTab {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.label("Default");
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

struct CanvasTab {
    canvas_zoom: f32,
    canvas_offset: Vec2,
}

impl CanvasTab {
    fn new() -> Self {
        Self {
            canvas_zoom: 1.0,
            canvas_offset: Vec2::new(0.0, 0.0),
        }
    }
}
impl BaseTab for CanvasTab {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let response = ui.allocate_response(ui.available_size(), Sense::drag());

        if response.dragged() {
            self.canvas_offset += response.drag_delta();
        }

        if response.hovered() {
            let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            let zoom_factor = 1.01;

            if scroll_delta > 0.0 {
                self.canvas_zoom *= zoom_factor;
            } else if scroll_delta < 0.0 {
                self.canvas_zoom /= zoom_factor;
            }
        }

        let rect = response.rect;

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, rect.size() / self.canvas_zoom),
            rect.translate(self.canvas_offset + (rect.size() / 2.0)),
        );

        let painter = ui.painter();
        let color = egui::Color32::GRAY;
        for i in -10..=10 {
            let i_f = i as f32 * 1.0;
            let start = to_screen * Pos2::new(i_f, -10.0);
            let end = to_screen * Pos2::new(i_f, 10.0);
            painter.line_segment([start, end], (1.0, color));

            let start = to_screen * Pos2::new(-10.0, i_f);
            let end = to_screen * Pos2::new(10.0, i_f);
            painter.line_segment([start, end], (1.0, color));
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct FileTab {
    filename: String,
}

impl FileTab {
    fn new(filename: String) -> Self {
        FileTab { filename }
    }
}

impl BaseTab for FileTab {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("HI I AM {}", self.filename));
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct FileExplorerTab;
impl BaseTab for FileExplorerTab {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl FileExplorerTab {
    fn new() -> Self {
        FileExplorerTab {}
    }
}

struct TerminalTab;
impl BaseTab for TerminalTab {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl TerminalTab {
    fn new() -> Self {
        TerminalTab {}
    }
}

struct BoardInfoTab;
impl BaseTab for BoardInfoTab {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl BoardInfoTab {
    fn new() -> Self {
        BoardInfoTab {}
    }
}

struct SettingsTab {
    should_random_colorscheme: bool,
    should_example_colorscheme: bool,
}
impl SettingsTab {
    fn new() -> Self {
        SettingsTab {
            should_random_colorscheme: false,
            should_example_colorscheme: false,
        }
    }
}

impl BaseTab for SettingsTab {
    fn draw(&mut self, ui: &mut egui::Ui) {
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
}

struct WindowContext {
    tabs: HashMap<String, Box<dyn BaseTab>>,
}

impl WindowContext {
    fn new() -> Self {
        let mut tabs: HashMap<String, Box<dyn BaseTab>> = HashMap::new();

        tabs.insert("Canvas".to_string(), Box::new(CanvasTab::new()));
        tabs.insert("Settings".to_string(), Box::new(SettingsTab::new()));
        tabs.insert(
            "File Explorer".to_string(),
            Box::new(FileExplorerTab::new()),
        );
        tabs.insert("Terminal".to_string(), Box::new(TerminalTab::new()));

        let filename = "main.rs".to_string();
        tabs.insert(filename.clone(), Box::new(FileTab::new(filename.clone())));

        Self { tabs }
    }
}

impl egui_dock::TabViewer for WindowContext {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        if let Some(tab) = self.tabs.get_mut(tab) {
            tab.draw(ui);
        }
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        self.tabs.remove(_tab);
        true
    }

    fn closeable(&mut self, _tab: &mut String) -> bool {
        if _tab == "Canvas" {
            false
        } else {
            true
        }
    }
}

pub struct MainWindow {
    tree: DockState<String>,
    context: WindowContext,
    keybindings: Keybindings,
    colorschemes: colorschemes::colorschemes,
}

impl Default for MainWindow {
    fn default() -> Self {
        let mut tree = DockState::new(vec![
            "Canvas".to_owned(),
            "Editor".to_owned(),
            "Settings".to_owned(),
        ]);

        let [a, b] = tree.main_surface_mut().split_left(
            NodeIndex::root(),
            0.3,
            vec!["File Explorer".to_owned()],
        );

        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.7, vec!["Terminal".to_owned()]);

        let context = WindowContext::new();
        let keybindings = Keybindings::new();
        let colorschemes = colorschemes::colorschemes::default();
        Self {
            tree,
            context,
            keybindings,
            colorschemes,
        }
    }
}

impl MainWindow {
    // /// Called once before the first frame.
    // pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    //     // This is also where you can customize the look and feel of egui using
    //     // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

    //     // Load previous app state (if any).
    //     // Note that you must enable the `persistence` feature for this to work.
    //     if let Some(storage) = cc.storage {
    //         return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
    //     }

    //     Default::default()
    // }
    pub fn display_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Settings").clicked() {
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    for tab_name in OPENABLE_TABS {
                        let is_open = self.context.tabs.contains_key(*tab_name);
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
                self.context
                    .tabs
                    .insert(tab_name.clone(), Box::new(SettingsTab::new()));
            }
            "Canvas" => {
                self.context
                    .tabs
                    .insert(tab_name.clone(), Box::new(CanvasTab::new()));
            }
            "Terminal" => {
                self.context
                    .tabs
                    .insert(tab_name.clone(), Box::new(TerminalTab));
            }
            "File Explorer" => {
                self.context
                    .tabs
                    .insert(tab_name.clone(), Box::new(FileExplorerTab));
            }
            "Board Info" => {
                self.context
                    .tabs
                    .insert(tab_name.clone(), Box::new(BoardInfoTab));
            }
            _ => {}
        }
        self.tree.push_to_focused_leaf(tab_name);
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.display_menu(ctx, _frame);

        // if self.keybindings.is_pressed(ctx, "open_settings") {
        //     // (only if settings is closed)
        //     if !self.context.tabs.contains_key("Settings") {
        //         self.add_tab("Settings".to_string());
        //     }
        // }

        if self.context.tabs.contains_key("Settings") {
            //make sure settings tab gets current context
            //need help with this line
            let settings_tab = self
                .context
                .tabs
                .get_mut("Settings")
                .unwrap()
                .as_any_mut()
                .downcast_mut::<SettingsTab>()
                .unwrap();
            if settings_tab.should_random_colorscheme == true {
                self.colorschemes
                    .set_color_scheme(&ctx, &colorschemes::colorschemes::get_random_color_scheme());
                settings_tab.should_random_colorscheme = false;
            } else if settings_tab.should_example_colorscheme == true {
                self.colorschemes.set_color_scheme(&ctx, &100);
                settings_tab.should_example_colorscheme = false;
            }
        }

        // if self.keybindings.is_pressed(ctx, "test_a") {
        //     println!("Test A!");
        // }

        // if self.keybindings.is_pressed(ctx, "test_b") {
        //     println!("Test B!");
        // }

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_leaf_collapse_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show(ctx, &mut self.context);
    }
}

// /// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
// pub struct TemplateApp {
//     // Example stuff:
//     label: String,

//     #[serde(skip)] // This how you opt-out of serialization of a field
//     value: f32,
// }

// impl Default for TemplateApp {
//     fn default() -> Self {
//         Self {
//             // Example stuff:
//             label: "Hello World!".to_owned(),
//             value: 2.7,
//         }
//     }
// }

// impl TemplateApp {
//     /// Called once before the first frame.
//     pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
//         // This is also where you can customize the look and feel of egui using
//         // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

//         // Load previous app state (if any).
//         // Note that you must enable the `persistence` feature for this to work.
//         if let Some(storage) = cc.storage {
//             return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
//         }

//         Default::default()
//     }
// }

// impl eframe::App for TemplateApp {
//     /// Called by the frame work to save state before shutdown.
//     fn save(&mut self, storage: &mut dyn eframe::Storage) {
//         eframe::set_value(storage, eframe::APP_KEY, self);
//     }

//     /// Called each time the UI needs repainting, which may be many times per second.
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
//         // For inspiration and more examples, go to https://emilk.github.io/egui

//         egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
//             // The top panel is often a good place for a menu bar:

//             egui::menu::bar(ui, |ui| {
//                 // NOTE: no File->Quit on web pages!
//                 let is_web = cfg!(target_arch = "wasm32");
//                 if !is_web {
//                     ui.menu_button("File", |ui| {
//                         if ui.button("Quit").clicked() {
//                             ctx.send_viewport_cmd(egui::ViewportCommand::Close);
//                         }
//                     });
//                     ui.add_space(16.0);
//                 }

//                 egui::widgets::global_theme_preference_buttons(ui);
//             });
//         });

//         egui::CentralPanel::default().show(ctx, |ui| {
//             // The central panel the region left after adding TopPanel's and SidePanel's
//             ui.heading("eframe template");

//             ui.horizontal(|ui| {
//                 ui.label("Write something: ");
//                 ui.text_edit_singleline(&mut self.label);
//             });

//             ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
//             if ui.button("Increment").clicked() {
//                 self.value += 1.0;
//             }

//             ui.separator();

//             ui.add(egui::github_link_file!(
//                 "https://github.com/emilk/eframe_template/blob/main/",
//                 "Source code."
//             ));

//             ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
//                 powered_by_egui_and_eframe(ui);
//                 egui::warn_if_debug_build(ui);
//             });
//         });
//     }
// }

// fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
//     ui.horizontal(|ui| {
//         ui.spacing_mut().item_spacing.x = 0.0;
//         ui.label("Powered by ");
//         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
//         ui.label(" and ");
//         ui.hyperlink_to(
//             "eframe",
//             "https://github.com/emilk/egui/tree/master/crates/eframe",
//         );
//         ui.label(".");
//     });
// }