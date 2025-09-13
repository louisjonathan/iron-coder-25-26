use crate::app::SharedState;

pub trait BaseTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.label("Default");
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn as_any(&self) -> &dyn std::any::Any;
}

// struct SampleTab {
// }

// impl SampleTab {
//     fn default() -> Self {
//         Self {
//         }
//     }
// }

// impl BaseTab for SampleTab {
//     fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
//     }

//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         self
//     }
// }
