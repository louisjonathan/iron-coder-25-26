// This is the example from https://github.com/Adanos020/egui_dock/blob/main/examples/hello.rs
// Modified for the purposes of Iron Coder https://github.com/shulltronics/iron-coder

use eframe::egui::{Pos2, Rect, Sense, Ui, Vec2};
use eframe::{egui, NativeOptions};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use emath::{self};

use std::collections::HashMap;

static OPENABLE_TABS: &'static [&'static str] = &[
    "Settings",
    "Canvas",
    "File Explorer",
    "Terminal",
    "Board Info",
];

pub fn main() -> eframe::Result<()> {
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

struct FileExplorerTab;
impl BaseTab for FileExplorerTab {}
impl FileExplorerTab {
    fn new() -> Self {
        FileExplorerTab {}
    }
}

struct TerminalTab;
impl BaseTab for TerminalTab {}
impl TerminalTab {
    fn new() -> Self {
        TerminalTab {}
    }
}

struct BoardInfoTab;
impl BaseTab for BoardInfoTab {}
impl BoardInfoTab {
    fn new() -> Self {
        BoardInfoTab {}
    }
}

struct SettingsTab;
impl SettingsTab {
    fn new() -> Self {
        SettingsTab {}
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
    tabs: HashMap<String, Box<dyn BaseTab>>,
}

impl MyContext {
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

struct MyApp {
    tree: DockState<String>,
    context: MyContext,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = DockState::new(vec![
            "Canvas".to_owned(),
            "main.rs".to_owned(),
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

        let context = MyContext::new();

        Self { tree, context }
    }
}

impl MyApp {
    pub fn display_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
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
                    .insert(tab_name.clone(), Box::new(SettingsTab));
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.display_menu(ctx, _frame);

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_leaf_collapse_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show(ctx, &mut self.context);
    }
}
