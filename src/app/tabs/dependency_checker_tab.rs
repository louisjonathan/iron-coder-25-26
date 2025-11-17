use crate::app::SharedState;
use crate::app::tabs::base_tab::BaseTab;
use crate::dependencies::checker::DependencyChecker;

pub struct DependencyCheckerTab {
    checker: DependencyChecker,
}

impl DependencyCheckerTab {
    pub fn new() -> Self {
        let mut checker = DependencyChecker::new();
        checker.check_all();
        
        Self {
            checker,
        }
    }
    
    fn refresh(&mut self) {
        self.checker.check_all();
    }
}

impl BaseTab for DependencyCheckerTab {
    fn draw(&mut self, ui: &mut egui::Ui, _state: &mut SharedState) {
        ui.heading("Dependency Checker");
        ui.separator();
        
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("cargo-generate:");
                ui.label(self.checker.cargo_generate.status_text());
            });
            
            ui.horizontal(|ui| {
                ui.label("espflash:");
                ui.label(self.checker.espflash.status_text());
            });
            
            ui.horizontal(|ui| {
                ui.label("ravedude:");
                ui.label(self.checker.ravedude.status_text());
            });
            
            ui.separator();
            
            if ui.button("Refresh").clicked() {
                self.refresh();
            }
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}