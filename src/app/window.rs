use crate::app::SharedState;
use crate::board::{Board, get_boards};
use super::tabs::*;
use crate::app::colorschemes::colorscheme;
use eframe::egui::{Ui};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

#[derive(Default)]
struct NewProjectDialog {
    name: String,
    path: String,
    selected_board_index: usize,
}

impl NewProjectDialog {
    fn reset(&mut self) {
        self.name.clear();
        self.path.clear();
        self.selected_board_index = 0;
    }
}

static OPENABLE_TABS: &'static [&'static str] = &[
    "Settings",
    "Canvas",
    "File Explorer",
    "Terminal",
    "Board Info",
	"Debug"
];

struct WindowContext<'a> {
    tabs: &'a mut HashMap<String, Box<dyn BaseTab>>,
    state: &'a mut SharedState,
    active_tab: &'a mut Option<String>,
}

impl<'a> egui_dock::TabViewer for WindowContext<'a> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        // Is FileTab? and change s made?
        if let Some(file_tab) = self.tabs.get(tab) {
            if let Some(file_tab) = file_tab.as_any().downcast_ref::<FileTab>() {
                if !file_tab.is_synced() {
                    return format!("● {}", tab).into();
                }
            }
        }
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        // Check if the pointer is over this tab's area and there was a click
        let rect = ui.max_rect();
        if ui.ctx().input(|i| i.pointer.any_click()) && 
           ui.ctx().input(|i| i.pointer.hover_pos().map_or(false, |pos| rect.contains(pos))) {
            *self.active_tab = Some(tab.clone());
        }
        
        if let Some(tab_content) = self.tabs.get_mut(tab) {
            tab_content.draw(ui, self.state);
        }
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        self.tabs.remove(_tab);
        true
    }

    fn closeable(&mut self, _tab: &mut String) -> bool {
        if _tab == "Canvas" || _tab == "File Explorer" {
            false
        } else {
            true
        }
    }
}

pub struct MainWindow {
    tree: DockState<String>,
    tabs: HashMap<String, Box<dyn BaseTab>>,
    state: SharedState,
    active_tab: Option<String>,
    show_new_project_dialog: bool,
    new_project_dialog: NewProjectDialog,
}

impl Default for MainWindow {
    fn default() -> Self {
        let mut tree = DockState::new(vec![
            "Canvas".to_owned(),
            "Settings".to_owned(),
        ]);

        let [a, b] = tree.main_surface_mut().split_left(
            NodeIndex::root(),
            0.3,
            vec!["Board Info".to_owned(), "File Explorer".to_owned()],
        );

        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.7, vec!["Terminal".to_owned()]);

        let mut tabs: HashMap<String, Box<dyn BaseTab>> = HashMap::new();

        tabs.insert("Board Info".to_string(), Box::new(BoardInfoTab::new()));
        tabs.insert("Canvas".to_string(), Box::new(CanvasTab::new()));
        tabs.insert("Settings".to_string(), Box::new(SettingsTab::new()));
        tabs.insert(
            "File Explorer".to_string(),
            Box::new(FileExplorerTab::new()),
        );
        tabs.insert("Terminal".to_string(), Box::new(TerminalTab::new()));

        let state= SharedState::default();

        Self {
            tree: tree,
            tabs: tabs,
            state: SharedState::default(),
            active_tab: None,
            show_new_project_dialog: false,
            new_project_dialog: NewProjectDialog::default(),
        }
    }
}

impl MainWindow {
    pub fn display_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if(!self.state.did_activate_colorscheme){
                self.state.did_activate_colorscheme = true;
                let name = self.state.colorschemes.name.clone();
                self.state.colorschemes.try_use_colorscheme(ui, &name);
            }
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Project").clicked() {
                        self.show_new_project_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("Open Project").clicked() {
                        self.open_project();
                        ui.close_menu();
                    }
                    if ui.button("Save Project").clicked() {
                        match self.state.project.save() {
                            Ok(_) => println!("Save successful."),
                            Err(e) => println!("Save failed: {}", e),
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Open").clicked() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        self.save_current_file();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Settings").clicked() {
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    for tab_name in OPENABLE_TABS {
                        let is_open = self.tabs.contains_key(*tab_name);
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
                ui.menu_button("Build", |ui| {
                    if ui.button("Build Project").clicked() {
                        // self.state.project.build(ctx);
                        ui.close_menu();
                    }
                    if ui.button("Flash to Board").clicked() {
                        self.state.load_to_board();
                        ui.close_menu();
                    }
                });
            });
        });
    }

    pub fn add_tab(&mut self, tab_name: String) {
        match tab_name.as_str() {
            "Settings" => {
                self.tabs
                    .insert(tab_name.clone(), Box::new(SettingsTab::new()));
            }
            "Canvas" => {
                self.tabs
                    .insert(tab_name.clone(), Box::new(CanvasTab::new()));
            }
            "Terminal" => {
                self.tabs
                    .insert(tab_name.clone(), Box::new(TerminalTab::new()));
            }
            "File Explorer" => {
                self.tabs
                    .insert(tab_name.clone(), Box::new(FileExplorerTab::new()));
            }
            "Board Info" => {
                self.tabs.insert(
                    tab_name.clone(),
                    Box::new(BoardInfoTab::new()),
                );
            }
			"Debug" => {
				self.tabs
                    .insert(tab_name.clone(), Box::new(DebugTab{}));
			}
            _ => {}
        }
        self.tree.push_to_focused_leaf(tab_name);
    }

    fn save_current_file(&mut self) {
        // Use the tracked active tab
        if let Some(active_tab_name) = &self.active_tab.clone() {
            if let Some(tab) = self.tabs.get_mut(active_tab_name) {
                if let Some(file_tab) = tab.as_any_mut().downcast_mut::<FileTab>() {
                    match file_tab.save() {
                        Ok(()) => {
                            println!("File '{}' saved successfully", active_tab_name);
                        }
                        Err(e) => {
                            println!("Error saving file '{}': {}", active_tab_name, e);
                        }
                    }
                } else {
                    println!("Current tab '{}' is not a file tab", active_tab_name);
                }
            } else {
                println!("Could not find active tab '{}'", active_tab_name);
            }
        } else {
            println!("No active tab found");
        }
    }

    fn get_active_tab_id(&self) -> Option<String> {
        self.active_tab.clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_file_dialog(&mut self) {
        // list of allowable files to open
        if let Some(file_path) = FileDialog::new()
            .add_filter("Supported files", &["rs", "json", "txt"])
            .add_filter("Rust files", &["rs"])
            .add_filter("JSON files", &["json"])
            .add_filter("Text files", &["txt"])
            .pick_file()
        {
            self.open_file(&file_path);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn open_file_dialog(&mut self) {
        println!("Blocking file opening because of wasm.");
    }

    fn open_project(&mut self) {
        match self.state.project.open(&self.state.known_boards) {
            Ok(()) => {
                self.refocus_file_explorer_to_project();
                println!("Project opened successfully.");
            }
            Err(e) => {
                println!("Failed to open project.");
            }
        }
    }

    fn open_file(&mut self, file_path: &Path) {
        let tab_name = file_path.display().to_string();
        
        if self.tabs.contains_key(&tab_name) {
            println!("File '{}' is already open", tab_name);
            return;
        }

        let mut file_tab = FileTab::default();
        match file_tab.load_from_file(file_path) {
            Ok(()) => {
                self.tabs.insert(tab_name.clone(), Box::new(file_tab));
                
                self.add_file_tab_intelligently(tab_name.clone());
                
                println!("File '{}' opened successfully", tab_name);
            }
            Err(e) => {
                println!("Error opening file '{}': {}", tab_name, e);
            }
        }
    }

    fn add_file_tab_intelligently(&mut self, tab_name: String) {
        self.tree.push_to_focused_leaf(tab_name);
    }

    fn is_file_tab(&self, tab_name: &str) -> bool {
        // list of allowable files to open
        tab_name.contains('.') && (
            tab_name.ends_with(".rs") || 
            tab_name.ends_with(".json") || 
            tab_name.ends_with(".txt") ||
            tab_name.contains('/') ||  // for unix
            tab_name.contains('\\') // for windows
        )
    }

    fn refocus_file_explorer_to_project(&mut self) {
        if let Some(file_explorer_tab) = self.tabs.get_mut("File Explorer") {
            if let Some(file_explorer) = file_explorer_tab.as_any_mut().downcast_mut::<FileExplorerTab>() {
                if let Some(project_location) = self.state.project.get_location_path() {
                    file_explorer.set_root_dir(project_location);
                }
            }
        }
    }

    fn display_new_project_dialog(&mut self, ctx: &egui::Context) {
        let mut should_create_project = false;
        let mut should_close_dialog = false;
        
        egui::Window::new("New Project")
            .open(&mut self.show_new_project_dialog)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Create a new Iron Coder project:");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Project Name:");
                    ui.text_edit_singleline(&mut self.new_project_dialog.name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Project Path:");
                    ui.text_edit_singleline(&mut self.new_project_dialog.path);
                    
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Browse...").clicked() {
                        if let Some(folder) = FileDialog::new().pick_folder() {
                            self.new_project_dialog.path = folder.display().to_string();
                        }
                    }
                    
                    #[cfg(target_arch = "wasm32")]
                    if ui.button("Browse...").clicked() {
                        // TODO: Add wasm32 support for folder picking
                        ui.label("Folder picking not supported in web version");
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Main Board:");
                    let main_boards: Vec<_> = self.state.known_boards.iter()
                        .filter(|b| b.borrow().is_main_board())
                        .collect();
                    
                    if !main_boards.is_empty() {
                        let selected_board_name = main_boards.get(self.new_project_dialog.selected_board_index)
                            .map(|b| b.borrow().get_name().to_owned())
                            .unwrap_or("Unknown".to_string());
                        
                        egui::ComboBox::from_label("")
                            .selected_text(selected_board_name)
                            .show_ui(ui, |ui| {
                                for (i, board) in main_boards.iter().enumerate() {
                                    ui.selectable_value(&mut self.new_project_dialog.selected_board_index, i, board.borrow().get_name());
                                }
                            });
                    } else {
                        ui.label("No main boards available");
                    }
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Create Project").clicked() {
                        should_create_project = true;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        should_close_dialog = true;
                    }
                });
            });
            
        if should_create_project {
            self.create_new_project();
        }
        
        if should_close_dialog {
            self.show_new_project_dialog = false;
            self.new_project_dialog.reset();
        }
    }

    fn create_new_project(&mut self) {
        if self.new_project_dialog.name.is_empty() {
            return;
        }
        
        if self.new_project_dialog.path.is_empty() {
            return;
        }
        
        // Create a new project
        let mut new_project = crate::project::Project::default();
        new_project.borrow_name().clone_from(&self.new_project_dialog.name);
        
        // select board for the project
        if self.new_project_dialog.selected_board_index < self.state.known_boards.len() {
            let main_boards: Vec<_> = self.state.known_boards.iter()
                .filter(|b| b.borrow().is_main_board())
                .collect();
            
            if self.new_project_dialog.selected_board_index < main_boards.len() {
                let board = main_boards[self.new_project_dialog.selected_board_index].clone();
                new_project.add_board(&board);
            }
        }
        
        // Set the location and save
        let project_path = PathBuf::from(&self.new_project_dialog.path);
        
        // create the project by saving it to the specified location
        let project_folder = project_path.join(&self.new_project_dialog.name);
        
        match std::fs::create_dir_all(&project_folder) {
            Ok(()) => {
                // Set the location for saving
                new_project.set_location(project_folder);
                
                // .ironcoder.toml
                match new_project.save() {
                    Ok(()) => {
                        // Get cargo-generate template
                        if new_project.main_board.is_some() {
                            match new_project.generate_cargo_template() {
                                Ok(()) => {
                                    println!("Project template generated.");
                                }
                                Err(e) => {
                                    println!("Is cargo-generate installed? Try: cargo install cargo-generate");
                                }
                            }
                        }
                        
                        // open the project
                        let project_location = new_project.get_location_path();
                        match self.state.project.load_from(&project_location.unwrap(), &self.state.known_boards) {
                            Ok(()) => {
                                self.refocus_file_explorer_to_project();
                                self.show_new_project_dialog = false;
                                let project_name = self.new_project_dialog.name.clone();
                                self.new_project_dialog.reset();
                                println!("Project '{}' created and opened.", project_name);
                            }
                            Err(e) => {
                                println!("Project created but failed to open: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error creating project: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error creating project directory: {}", e);
            }
        }
    }

    

}

impl eframe::App for MainWindow {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("Exiting application, saving settings...");
        self.state.save_settings();
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update project terminal output and forward to Terminal tab
        // self.state.project.update_terminal_output();
        // let build_output = self.state.project.get_terminal_output();
        // if !build_output.is_empty() {
        //     if let Some(terminal_tab) = self.tabs.get_mut("Terminal") {
        //         if let Some(terminal) = terminal_tab.as_any_mut().downcast_mut::<TerminalTab>() {
        //             terminal.append_build_output(build_output);
        //         }
        //     }
        //     self.state.project.clear_terminal_output();
        // }

        
        self.display_menu(ctx, _frame);

        if self.show_new_project_dialog {
            self.display_new_project_dialog(ctx);
        }

        if self.state.keybindings.is_pressed(ctx, "save_file") {
            self.save_current_file();
        }

        if self.state.keybindings.is_pressed(ctx, "close_tab") {
            // close tab keybind for Jon... (once I figure out how to reliably find the current tab)
            println!("Close tab bind pressed...");
        }

        // wait for file picker
        if let Some(file_path) = self.state.requested_file_to_open.take() {
            self.open_file(&file_path);
        }

        let mut context = WindowContext {
            tabs: &mut self.tabs,
            state: &mut self.state,
            active_tab: &mut self.active_tab,
        };

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_leaf_collapse_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show(ctx, &mut context);
    }
}