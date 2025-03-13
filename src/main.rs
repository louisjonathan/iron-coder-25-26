// This is the example from https://github.com/Adanos020/egui_dock/blob/main/examples/hello.rs
// Modified for the purposes of Iron Coder https://github.com/shulltronics/iron-coder

use eframe::{egui, NativeOptions};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use eframe::egui::{Pos2, Rect, Sense, Ui, Vec2};
use emath::{self};

use std::collections::HashMap;

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

trait BaseTab {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.label("Default");
    }
}

struct CanvasTab {
    canvas_zoom: f32,
    canvas_offset: Vec2,
}

impl CanvasTab {
    fn new(zoom: f32, offset: Vec2) -> Self {
        Self {
            canvas_zoom: zoom,
            canvas_offset: offset,
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
}

struct SettingsTab;

impl SettingsTab {
    fn new() -> Self {
        SettingsTab { }
    }
}

impl BaseTab for SettingsTab {
    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.label("Random setting 1");
        let mut s1_value = 50.0;
        ui.add(egui::Slider::new(&mut s1_value, 0.0..=100.0).text("Slider 1"));
        if ui.button("Save settings").clicked() {
            println!("Settings saved!");
        }
    }
}

struct MyContext {
    tabs: HashMap<String, Box<dyn BaseTab>>, // these are tabs like Settings, Canvas, etc that can be reopened
}

impl MyContext {
    fn new() -> Self {
        let mut tabs: HashMap<String, Box<dyn BaseTab>> = HashMap::new();

        tabs.insert("Canvas".to_string(), Box::new(CanvasTab::new(1.0, Vec2::new(0.0, 0.0))));
        tabs.insert("Settings".to_string(), Box::new(SettingsTab::new()));

        let filename = "main.rs".to_string();
        tabs.insert(filename.clone(), Box::new(FileTab::new(filename.clone())));


        Self {
            tabs,
        }
    }
}

impl egui_dock::TabViewer for MyContext {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        if let Some(tab) = self.tabs.get_mut(tab) {
            tab.draw(ui);
        }
    }

    fn closeable(&mut self, _tab: &mut String) -> bool {
        if _tab == "Canvas" {
            false
        } else {
            true
        }
    }
}

struct MyApp {
    tree: DockState<String>,
    display_settings: bool,
    context: MyContext,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = DockState::new(vec!["Canvas".to_owned(), "main.rs".to_owned(), "Settings".to_owned()]);

        // You can modify the tree before constructing the dock
        let [a, b] =
            tree.main_surface_mut()
                .split_left(NodeIndex::root(), 0.3, vec!["File Explorer".to_owned()]);
        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.7, vec!["Terminal".to_owned()]);
        let [_, _] = tree
            .main_surface_mut()
            .split_below(b, 0.5, vec!["Board Info".to_owned()]);

        let context = MyContext::new();

        Self {
            tree,
            display_settings: false,
            context,

        }
    }
}

impl MyApp {
    pub fn display_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            display_settings,
            ..
        } = self;
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Settings").clicked() {
                        *display_settings = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    pub fn display_settings_window(&mut self, ctx: &egui::Context) {
        let Self {
            display_settings,
            ..
        } = self;

        if *display_settings {
            let window_response = egui::Window::new("Settings")
            .open(display_settings)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .show(ctx, |ui| {
                ui.heading("Random setting 1");
            });
            window_response.unwrap().response.layer_id.order = egui::Order::Foreground;
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.display_menu(ctx, _frame);

        self.display_settings_window(ctx);

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_leaf_collapse_buttons(false)
            .show(ctx, &mut self.context);
    }
}