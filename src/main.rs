#![allow(warnings)]
use egui::FontDefinitions;
use iron_coder::MainWindow;
use egui_extras::install_image_loaders;

pub fn main() -> eframe::Result<()> {
    let icon_data = include_bytes!("../assets/application-icon/icon.png");
    let icon = eframe::icon_data::from_png_bytes(icon_data).ok();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size((400.0, 200.0))
            .with_icon(icon.unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "IRON CODER",
        options,
        Box::new(|cc| {
        install_image_loaders(&cc.egui_ctx); // <-- call here
        Ok(Box::<MainWindow>::default())
        }),
    )?;

    Ok(())
}
