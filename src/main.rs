#![allow(warnings)]
use egui_dock_testing::app::MainWindow;
use egui_extras::install_image_loaders;

#[cfg(not(target_arch = "wasm32"))]
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

///from https://github.com/emilk/eframe_template/blob/main/src/main.rs
#[cfg(target_arch = "wasm32")]
pub fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    use eframe::web_sys;
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");
        ///@todo Give the_canvas_id a reasonable name
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| Ok(Box::<MainWindow>::default())),
            )
            .await;
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> ERROR: Crashed during removal of loading text and spinner </p>",
                    );
                    panic!("Failed to start eframe {e:?}");
                }
            }
        }
    });
}
