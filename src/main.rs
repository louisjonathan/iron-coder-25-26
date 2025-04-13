use eframe::NativeOptions;
use egui_dock_testing::IronCoderApp;

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<IronCoderApp>::default())),
    )
}
