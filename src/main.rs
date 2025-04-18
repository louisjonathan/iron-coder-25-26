#![allow(warnings)]
use egui_dock_testing::app::MainWindow;

#[cfg(not(target_arch = "wasm32"))]
pub fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size((400.0, 200.0)),
        ..Default::default()
    };

    eframe::run_native(
        "IRON CODER",
        options,
        Box::new(|_cc| Ok(Box::<MainWindow>::default())),
    )
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
