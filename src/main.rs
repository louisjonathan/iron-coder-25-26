use eframe::NativeOptions;
use egui_dock_testing::IronCoderApp;

#[cfg(not(target_arch = "wasm32"))]
pub fn main() -> eframe::Result<()> {
    use iron_coder::MainWindow;

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<MainWindow>::default())),
    )
}