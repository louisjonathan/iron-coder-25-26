// This is the example from https://github.com/Adanos020/egui_dock/blob/main/examples/simple.rs
// Modified for the purposes of Iron Coder https://github.com/shulltronics/iron-coder

use eframe::{egui, NativeOptions};
use egui_dock::{DockArea, DockState, NodeIndex, Style};

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct TabViewer {}

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {tab}"));
    }
}

struct MyApp {
    tree: DockState<String>,
    display_settings: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = DockState::new(vec!["tab1".to_owned(), "tab2".to_owned()]);

        // You can modify the tree before constructing the dock
        let [a, b] =
            tree.main_surface_mut()
                .split_left(NodeIndex::root(), 0.3, vec!["tab3".to_owned()]);
        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.7, vec!["tab4".to_owned()]);
        let [_, _] = tree
            .main_surface_mut()
            .split_below(b, 0.5, vec!["tab5".to_owned()]);

        Self {
            tree,
            display_settings: false,
        }
    }
}

impl MyApp {
    pub fn display_menu(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
            .show(ctx, &mut TabViewer {});
    }
}