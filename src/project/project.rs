//! Title: Iron Coder Project Module - Module
//! Description: This module contains the Project struct and its associated functionality.
#![allow(warnings)]
use log::{debug, info, warn};
use proc_macro2::extra;

// use std::error::Error;
use std::fs::{self, DirEntry, read_dir, read_to_string, write};
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use crate::app::connection_wizard::WizardType;
use crate::app::{CanvasBoard, CanvasConnection, CanvasProtocol, SharedState};
use crate::board::{BoardStandards, GPIODirection, get_boards};

use egui::Context;

use rfd::FileDialog;

use serde::{Deserialize, Serialize};

use crate::board::Board;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

use prettyplease;
use quote::quote;
use syn::{File, Item, ItemFn, Stmt};

use crate::board::pinout::RoleAssignment;

const PROJECT_FILE_NAME: &'static str = ".ironcoder.toml";

pub type Result = core::result::Result<(), ProjectIOError>;

#[non_exhaustive]
#[derive(Debug)]
pub enum ProjectIOError {
    FilePickerAborted,
    NoMainBoard,
    NoProjectTemplate,
    NoProjectDirectory,
    FilesystemError,
    LoadToTomlError,
}

/// A Project represents the highest level of Iron Coder, which contains
/// a main, programmable development board, a set of peripheral development boards,
/// and the project/source code directory
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Project {
    pub name: String,
    pub location: Option<PathBuf>,
    pub source_files: Vec<PathBuf>,

    #[serde(with = "rc_refcell_option")]
    pub main_board: Option<Rc<RefCell<CanvasBoard>>>,

    #[serde(with = "rc_refcell_vec")]
    pub peripheral_boards: Vec<Rc<RefCell<CanvasBoard>>>,

    #[serde(with = "rc_refcell_vec")]
    pub connections: Vec<Rc<RefCell<CanvasConnection>>>,

    /// Protocol groups (I2C, SPI, UART) that bundle multiple connections together
    #[serde(default)]
    pub protocol_groups: HashMap<Uuid, CanvasProtocol>,

    #[serde(skip)]
    pub board_map: HashMap<Uuid, Rc<RefCell<CanvasBoard>>>,

    #[serde(skip)]
    pub has_unsaved_changes: bool,
}

// backend functionality for Project struct
impl Project {
    // Helper function for printing both to logs and to built-in terminal
    fn info_logger(&mut self, msg: &str) {
        info!("{}", msg);
        let msg = msg.to_owned() + "\n";
    }

    pub fn borrow_name(&mut self) -> &mut String {
        return &mut self.name;
    }

    pub fn has_main_board(&self) -> bool {
        if let Some(_) = self.main_board {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_location(&self) -> String {
        if let Some(project_folder) = &self.location {
            // let s = project_folder.display().to_string();
            return project_folder.display().to_string().clone();
        } else {
            return String::from("");
        }
    }

    pub fn get_location_path(&self) -> Option<PathBuf> {
        self.location.clone()
    }

    pub fn set_location(&mut self, path: PathBuf) {
        self.location = Some(path);
    }

    /// file explorer should be refocused to the project directory
    pub fn should_refocus_file_explorer(&self) -> bool {
        self.location.is_some()
    }

    pub fn add_board(&mut self, board: &Rc<Board>) -> Option<Rc<RefCell<CanvasBoard>>> {
        match board.is_main_board() {
            true => {
                if self.has_main_board() {
                    info!("project already contains a main board! aborting.");
                    return None;
                } else {
                    if let Some(b) = CanvasBoard::new(&board) {
                        let b_ref = Rc::new(RefCell::new(b));
                        let extra_ref = b_ref.clone();
                        self.board_map
                            .insert(b_ref.borrow().id.clone(), b_ref.clone());
                        self.main_board = Some(b_ref);
                        self.mark_unsaved();
                        return Some(extra_ref);
                    }
                }
            }
            false => {
                if let Some(b) = CanvasBoard::new(&board) {
                    let b_ref = Rc::new(RefCell::new(b));
                    let extra_ref = b_ref.clone();
                    self.board_map
                        .insert(b_ref.borrow().id.clone(), b_ref.clone());
                    self.peripheral_boards.push(b_ref);
                    self.mark_unsaved();
                    return Some(extra_ref);
                }
            }
        }
        return None;
    }

    /// Populate the project board list via the app-wide 'known boards' list
    pub fn load_board_resources(&mut self, kb: &Vec<Rc<Board>>) {
        if let Some(b) = &self.main_board {
            let board_id = b.borrow().id;
            self.board_map.insert(board_id, b.clone());
            b.borrow_mut().init_refs(kb, &self);
        }
        for b in &self.peripheral_boards {
            let board_id = b.borrow().id;
            self.board_map.insert(board_id, b.clone());
            b.borrow_mut().init_refs(kb, &self);
        }
        for c in &self.connections {
            c.borrow_mut().init_refs(kb, &self);
        }
        // Initialize protocol group references
        for (_, protocol_group) in self.protocol_groups.iter_mut() {
            protocol_group.init_refs(&self.connections);
        }
    }

    /// This method will reload the project based on the current project location
    pub fn reload(&mut self, kb: &Vec<Rc<Board>>) -> Result {
        if let Some(location) = self.location.clone() {
            self.load_from(&location, kb)
        } else {
            Err(ProjectIOError::NoProjectDirectory)
        }
    }

    /// Load a project from a specified directory, and sync the board assets.
    pub fn load_from(&mut self, project_directory: &Path, kb: &Vec<Rc<Board>>) -> Result {
        let project_file = project_directory.join(PROJECT_FILE_NAME);
        let toml_str = match fs::read_to_string(project_file) {
            Ok(s) => s,
            Err(e) => {
                warn!("error reading project file: {:?}", e);
                return Err(ProjectIOError::FilesystemError);
            }
        };
        let p: Project = match toml::from_str(&toml_str) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to parse TOML: {}", e);
                return Err(ProjectIOError::LoadToTomlError);
            }
        };
        self.name = p.name;
        self.location = Some(project_directory.to_path_buf());
        self.main_board = p.main_board;
        self.peripheral_boards = p.peripheral_boards;
        self.connections = p.connections;
        self.protocol_groups = p.protocol_groups;
        self.has_unsaved_changes = false; // just loaded from disk therefore no changes
        self.load_board_resources(kb);
        self.find_source_files();
        self.update_toolchain_location();
        Ok(())
    }
    /// Determine platform for toolchain selection
    fn get_platform_toolchain_dir() -> &'static str {
        if cfg!(target_os = "linux") {
            "linux-x86_64"
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "macos-aarch64"
            } else {
                "macos-x86_64"
            }
        } else if cfg!(target_os = "windows") {
            "windows-x86_64"
        } else {
            "unknown"
        }
    }
    /// Select toolchain if necessary and update cargo config
    pub fn update_toolchain_location(&mut self) {
        if let Some(loc) = &self.location {
            if let Some(mut ide_path) = std::env::current_exe()
                .ok()
                .map(|p| p.to_path_buf().parent().unwrap().to_path_buf())
            {
                if ide_path.ends_with("debug") || ide_path.ends_with("release") {
                    println!(
                        "Detected debug/release build, adjusting toolchain path respectively."
                    );
                    ide_path = ide_path.parent().unwrap().parent().unwrap().to_path_buf();
                    // println!("IDE path: {}", ide_path.display());
                }

                if let Some(cpu_name) = self.main_board.as_ref().and_then(|b| {
                    let b = b.borrow();
                    b.board.cpu.clone()
                }) {
                    if cpu_name != "Microchip AVR" {
                        return;
                    }

                    let toolchain_name = "Arduino";
                    let os_str = Self::get_platform_toolchain_dir();
                    if os_str == "unknown" {
                        warn!("Unsupported OS for toolchain configuration.");
                        return;
                    }
                    let toolchain_bin = ide_path
                        .join("Redist")
                        .join(toolchain_name)
                        .join("toolchain")
                        .join(os_str)
                        .join("bin");

                    // Point to avr-gcc for build, avrdude for upload
                    let avr_gcc_path = if cfg!(windows) {
                        toolchain_bin.join("avr-gcc.exe")
                    } else {
                        toolchain_bin.join("avr-gcc")
                    };

                    let cargo_config_dir = loc.join(".cargo");
                    let cargo_config_file = cargo_config_dir.join("config.toml");

                    // Ensure .cargo directory exists
                    if let Err(e) = fs::create_dir_all(&cargo_config_dir) {
                        warn!("Failed to create .cargo directory: {}", e);
                        return;
                    }

                    let mut config = toml::value::Table::new();

                    // If config.toml exists, read and parse it
                    if cargo_config_file.exists() {
                        if let Ok(contents) = fs::read_to_string(&cargo_config_file) {
                            if let Ok(parsed) = contents.parse::<toml::Value>() {
                                if let Some(table) = parsed.as_table() {
                                    config = table.clone();
                                }
                            }
                        }
                    }

                    // TOML is weird, target is a section, and that section contains the key cfg(target_arch = "avr")
                    let target_section_key = "target";
                    let target_cfg_key = "cfg(target_arch = \"avr\")";

                    // Get or create the [target] section
                    let target_section = config
                        .entry(target_section_key.to_string())
                        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

                    if let toml::Value::Table(target_map) = target_section {
                        // Get or create the specific target config
                        let target_table = target_map
                            .entry(target_cfg_key.to_string())
                            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

                        if let toml::Value::Table(target) = target_table {
                            target.insert(
                                "linker".to_string(),
                                toml::Value::String(avr_gcc_path.to_string_lossy().to_string()),
                            );
                        }
                    }

                    // Write back to config.toml
                    let toml_str = toml::to_string_pretty(&config).unwrap_or_default();
                    if let Err(e) = fs::write(&cargo_config_file, toml_str) {
                        warn!("Failed to write toolchain config: {}", e);
                    }
                }
            }
        }
    }
    /// Prompt the user to select project directory to open
    pub fn open(&mut self, kb: &Vec<Rc<Board>>) -> Result {
        if let Some(project_directory) = FileDialog::new().pick_folder() {
            self.load_from(&project_directory, kb)
        } else {
            info!("project open aborted");
            Err(ProjectIOError::FilePickerAborted)
        }
    }

    /// Open a file dialog to select a project folder, and then call the save method
    /// TODO - make file dialog have default directory
    pub fn save_as(&mut self, create_containing_folder: bool) -> io::Result<()> {
        if let Some(mut project_folder) = FileDialog::new().pick_folder() {
            // if indicated, create a new folder for the project (with same name as project)
            if create_containing_folder {
                project_folder = project_folder.join(self.name.clone());
                fs::create_dir(project_folder.as_path())?;
            }
            // check if there is an existing .ironcoder.toml file that we might overwrite
            for entry in std::fs::read_dir(&project_folder).unwrap() {
                if entry.unwrap().file_name().to_str().unwrap() == PROJECT_FILE_NAME {
                    warn!(
                        "you might be overwriting an existing Iron Coder project! \
						Are you sure you wish to continue?"
                    );
                    return Ok(());
                }
            }
            self.location = Some(project_folder);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "project save aborted!",
            ));
        }
        self.save()
    }

    // TODO - have this save all project files, maybe, except the target directory -- FIXED (note: currently only saves all open tabs)
    pub fn save(&mut self) -> io::Result<()> {
        if self.location == None {
            info!("no project location, calling save_as...");
            self.save_as(true)
        } else {
            let project_folder = self.location.clone().unwrap();
            let project_file = project_folder.join(PROJECT_FILE_NAME);
            info!(
                "saving project file to {}",
                project_file.display().to_string()
            );

            match toml::to_string(self) {
                Ok(contents) => {
                    fs::write(project_file, contents)?;
                    // Mark as saved
                    self.has_unsaved_changes = false;
                }
                Err(e) => {
                    warn!("couldn't save project to toml file!! {:?}", e);
                }
            }

            // self.code_editor.save_all().unwrap_or_else(|_| warn!("error saving tabs!"));
            Ok(())
        }
    }

    pub fn mark_unsaved(&mut self) {
        self.has_unsaved_changes = true;
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    pub fn generate_cargo_template(&mut self) -> Result {
        if let Some(mb) = &self.main_board {
            if let Some(template_dir) = mb.borrow().board.get_template_dir() {
                let destination = self.get_location();

                let cmd = duct::cmd!(
                    "cargo",
                    "generate",
                    "--path",
                    template_dir.as_path().to_str().unwrap(),
                    "--name",
                    self.name.clone(),
                    "--destination",
                    destination.clone(),
                    "--init",
                );

                match cmd.run() {
                    Ok(output) => Ok(()),
                    Err(e) => Err(ProjectIOError::FilesystemError),
                }
            } else {
                return Err(ProjectIOError::NoProjectTemplate);
            }
        } else {
            return Err(ProjectIOError::NoMainBoard);
        }
    }

    pub fn find_source_files(&mut self) {
        if let Some(loc) = &self.location {
            let src_path = loc.join("src");
            let mut source_files = Vec::new();
            self.recursive_add_source(&src_path, &mut source_files);
            self.source_files = source_files;
        }
    }

    pub fn recursive_add_source(&mut self, path: &PathBuf, source_files: &mut Vec<PathBuf>) {
        if !path.exists() {
            return;
        }

        if path.is_dir() {
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                let child_path = entry.path();
                                self.recursive_add_source(&child_path, source_files);
                            }
                            Err(err) => {
                                eprintln!("Warning: Could not read entry in {:?}: {}", path, err);
                                continue;
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Warning: Could not read directory {:?}: {}", path, err);
                    return;
                }
            }
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            source_files.push(path.to_path_buf());
        }
    }

    pub fn remove_board(&mut self, board: &Rc<RefCell<CanvasBoard>>) {
        let b = board.borrow_mut();
        if b.board.is_main_board() {
            return;
        }
        for c in &b.connections {
            self.remove_connection(c);
        }
        self.peripheral_boards.retain(|c| !Rc::ptr_eq(c, board));
    }

    pub fn remove_connection(&mut self, connection: &Rc<RefCell<CanvasConnection>>) {
        let main_file = self.source_files.iter().find(|path| {
            path.file_name()
                .map(|name| name == "main.rs")
                .unwrap_or(false)
        });

        let conn = connection.borrow();

        if let Some(path) = main_file {
            self.remove_pin_from_source(path, &conn);
        }

        if let Some(group_id) = conn.protocol_group_id {
            if let Some(group) = self.protocol_groups.get_mut(&group_id) {
                group.connections.retain(|c| !Rc::ptr_eq(c, connection));
            }
        }

        drop(conn);

        self.connections.retain(|c| !Rc::ptr_eq(c, connection));
    }

    pub fn boards_iter(&self) -> impl Iterator<Item = &Rc<RefCell<CanvasBoard>>> {
        self.main_board.iter().chain(self.peripheral_boards.iter())
    }

    pub fn boards_iter_rev(&self) -> impl Iterator<Item = &Rc<RefCell<CanvasBoard>>> {
        self.peripheral_boards
            .iter()
            .rev()
            .chain(self.main_board.iter().rev())
    }

    pub fn connections_iter(&self) -> impl Iterator<Item = &Rc<RefCell<CanvasConnection>>> {
        self.connections.iter()
    }

    pub fn add_connection(&mut self, c: &Rc<RefCell<CanvasConnection>>) {
        self.connections.push(c.clone());

        let main_file = self.source_files.iter().find(|path| {
            path.file_name()
                .map(|name| name == "main.rs")
                .unwrap_or(false)
        });

        let conn = c.borrow();

        if let Some(path) = main_file {
            self.insert_pin_into_source(path, &conn);
        }
    }

    // ===== Protocol Group Management =====

    /// Add a protocol group to the project
    pub fn add_protocol_group(&mut self, group: CanvasProtocol) {
        self.protocol_groups.insert(group.id, group);
    }

    /// Remove a protocol group from the project
    pub fn remove_protocol_group(&mut self, group_id: &Uuid) {
        let main_file = self.source_files.iter().find(|path| {
            path.file_name()
                .map(|name| name == "main.rs")
                .unwrap_or(false)
        });

        if let Some(protocol) = self.protocol_groups.get(group_id) {
            if let Some(path) = main_file {
                self.remove_bus_from_source(path, &protocol.protocol_type);
            }
        }

        self.protocol_groups.remove(group_id);
    }

    /// Get a protocol group by ID
    pub fn get_protocol_group(&self, group_id: &Uuid) -> Option<&CanvasProtocol> {
        self.protocol_groups.get(group_id)
    }

    /// Get all connections that belong to a protocol group
    pub fn get_group_connections(&self, group_id: &Uuid) -> Vec<Rc<RefCell<CanvasConnection>>> {
        if let Some(group) = self.protocol_groups.get(group_id) {
            group.connections.clone()
        } else {
            vec![]
        }
    }

    pub fn add_connection_bus(&mut self, bus_type: &WizardType) {
        let main_file = self.source_files.iter().find(|path| {
            path.file_name()
                .map(|name| name == "main.rs")
                .unwrap_or(false)
        });

        if let Some(path) = main_file {
            self.insert_bus_into_source(path, bus_type);
        }
    }

    fn insert_bus_into_source(&self, path: &PathBuf, bus_type: &WizardType) {
        let marker = "INTERFACE_DEFINITIONS".to_string();

        let code = read_to_string(&path).unwrap();
        let mut output = Vec::new();
        let mut inserted = false;

        let Some(new_stmt_str) = self.generate_bus_statement(bus_type) else {
            return;
        };
        println!("HERES WHAT WIZ WANTS: {}", new_stmt_str);

        for line in code.lines() {
            output.push(line.to_string());

            if !inserted && line.contains(&marker) {
                let indent = line
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .collect::<String>();
                output.push(format!("{}{}", indent, new_stmt_str));
                inserted = true;
            }
        }
        let code = output.join("\n");
        write(path, code);
    }

    fn remove_bus_from_source(&self, path: &PathBuf, bus_type: &WizardType) {
        let Some(stmt_to_remove) = self.generate_bus_statement(bus_type) else {
            return;
        };

        let Ok(code) = read_to_string(&path) else {
            return;
        };

        let mut output = Vec::new();
        let lines: Vec<&str> = code.lines().collect();
        let mut skip_next = false;

        for line in lines {
            if skip_next {
                skip_next = false;
                continue;
            }

            let trimmed = line.trim();

            if stmt_to_remove.contains('\n') {
                let parts: Vec<&str> = stmt_to_remove.split('\n').collect();
                if parts.len() == 2 && trimmed == parts[0].trim() {
                    skip_next = true;
                    continue;
                }
            }

            if trimmed == stmt_to_remove.trim() {
                continue;
            }

            output.push(line.to_string());
        }

        let code = output.join("\n");
        write(path, code);
    }

    fn insert_pin_into_source(&self, path: &PathBuf, conn: &CanvasConnection) {
        let marker = "PIN_DEFINITIONS".to_string();

        let code = read_to_string(&path).unwrap();
        let mut output = Vec::new();
        let mut inserted = false;

        let Some(new_stmt_str) = self.generate_pin_statement(conn) else {
            return;
        };
        println!("HERES WHAT CONN WANTS {}", new_stmt_str);

        for line in code.lines() {
            output.push(line.to_string());

            if !inserted && line.contains(&marker) {
                let indent = line
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .collect::<String>();
                output.push(format!("{}{}", indent, new_stmt_str));
                inserted = true;
            }
        }
        let code = output.join("\n");
        write(path, code);
    }

    fn remove_pin_from_source(&self, path: &PathBuf, conn: &CanvasConnection) {
        let Some(stmt_to_remove) = self.generate_pin_statement(conn) else {
            return;
        };

        let Ok(code) = read_to_string(&path) else {
            return;
        };

        let mut output = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed == stmt_to_remove.trim() {
                continue;
            }

            output.push(line.to_string());
        }

        let code = output.join("\n");
        write(path, code);
    }

    fn generate_bus_statement(&self, bus_type: &WizardType) -> Option<String> {
        let Some(sb_rc) = &self.main_board else {
            return None;
        };
        let sb = &sb_rc.borrow().board;
        let fmt = match sb.get_board_standard() {
            Some(BoardStandards::Arduino) => match bus_type {
                WizardType::I2C => format!("let mut i2c = setup_i2c!(dp, sda, scl, 10_000);"),
                WizardType::SPI => format!("let mut i2c = setup_spi!(dp, sck, mosi, miso, ss);"),
                _ => return None,
            },
            Some(BoardStandards::ESP32) => match bus_type {
                WizardType::I2C => format!(
                    "let i2c_peripheral = peripherals.I2C0;\n    let mut i2c = setup_i2c!(i2c_peripheral, sda, scl, 10);"
                ),
                WizardType::SPI => format!(
                    "let spi_peripheral = peripherals.SPI2;\n    let mut spi = setup_spi!(spi_peripheral, sck, mosi, miso, 10);"
                ),
                _ => return None,
            },
            Some(BoardStandards::Feather) => match bus_type {
                WizardType::I2C => format!(
                    "let i2c_peripheral = pac.I2C1;\n    let mut i2c = setup_i2c!(pac, clocks, 100_000, i2c_peripheral, sda, scl);"
                ),
                WizardType::SPI => format!(
                    "let spi_peripheral = peripherals.SPI2;\n    let mut spi = setup_spi!(spi_peripheral, sck, mosi, miso, 10);"
                ),
                _ => return None,
            },
            _ => return None,
        };
        return Some(fmt);
    }

    fn generate_pin_statement(&self, conn: &CanvasConnection) -> Option<String> {
        let sb_rc = conn.get_start_board();
        let sb = &sb_rc.borrow().board;
        let eb_rc = conn.get_end_board()?;
        let eb = &eb_rc.borrow().board;

        let start_pin = conn.get_start_pin();
        let end_pin = conn.get_end_pin().unwrap();
        if let Some((pin_interface, pin_role)) = eb.get_peripheral_pin_interface(&end_pin) {
            if let Some(possible_pins) = sb.pinout.get_pins_from_role(&pin_interface) {
                if !possible_pins.contains(&start_pin) {
                    return None;
                };
            }
            let pin_alias = sb
                .pinout
                .get_pin_alias(&start_pin, &"GPIO".to_string())
                .unwrap_or_default();

            let conn_type = eb
                .pinout
                .get_pin_alias(&end_pin, &pin_interface)
                .unwrap_or_default()
                .to_lowercase();

            let var_name = format!("c_{}_to_{}", start_pin, end_pin);

            // println!("PARSING CONNECTION");

            let fmt = match sb.get_board_standard() {
                Some(BoardStandards::Arduino) => match pin_interface.as_str() {
                    "GPIO" => {
                        let pin_type_str = match pin_role.direction {
                            Some(GPIODirection::Input) => "pull_up_input",

                            Some(GPIODirection::Output) => "output",
                            _ => return None,
                        };
                        let mutability = if pin_type_str == "output" { "mut " } else { "" };
                        format!(
                            "let {}pin_{} = pins.{}.into_{}();",
                            mutability, var_name, pin_alias, pin_type_str
                        )
                    }
                    "SPI" => {
                        format!("let {} = pins.{}.into_output();", conn_type, pin_alias)
                    }
                    "I2C" => {
                        format!(
                            "let {} = pins.{}.into_floating_input().into_pull_up_input();",
                            conn_type, pin_alias
                        )
                    }
                    _ => return None,
                },
                Some(BoardStandards::ESP32) => match pin_interface.as_str() {
                    "GPIO" => match pin_role.direction {
                        Some(GPIODirection::Input) => format!(
                            "let pin_{} = Input::new(peripherals.{}, InputConfig::default().with_pull(Pull::Up));",
                            var_name, pin_alias
                        ),
                        Some(GPIODirection::Output) => format!(
                            "let mut pin_{} = Output::new(peripherals.{}, Level::High, OutputConfig::default());",
                            var_name, pin_alias
                        ),
                        _ => return None,
                    },
                    "I2C" | "SPI" => {
                        format!("let {} = peripherals.{};", conn_type, pin_alias)
                    }
                    _ => {
                        println!("COULDNT FIND INTERFACE {:?}", pin_interface);
                        return None;
                    }
                },
                Some(BoardStandards::Feather) => match pin_interface.as_str() {
                    "GPIO" => match pin_role.direction {
                        Some(GPIODirection::Input) => format!(
                            "let pin_{} = pins.{}.into_pull_up_input();",
                            var_name, pin_alias
                        ),
                        Some(GPIODirection::Output) => format!(
                            "let mut pin_{} = pins.{}.into_push_pull_output();",
                            var_name, pin_alias
                        ),
                        _ => return None,
                    },
                    "I2C" | "SPI" => {
                        format!("let {} = pins.{};", conn_type, pin_alias)
                    }
                    _ => {
                        println!("COULDNT FIND INTERFACE {:?}", pin_interface);
                        return None;
                    }
                },
                _ => return None,
            };
            return Some(fmt);
        }
        return None;
    }
}

pub mod rc_refcell_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::{cell::RefCell, rc::Rc};

    pub fn serialize<T, S>(v: &Vec<Rc<RefCell<T>>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Clone + Serialize,
        S: Serializer,
    {
        let plain: Vec<T> = v.iter().map(|c| c.borrow().clone()).collect();
        plain.serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<Rc<RefCell<T>>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let plain: Vec<T> = Vec::deserialize(deserializer)?;
        Ok(plain
            .into_iter()
            .map(|c| Rc::new(RefCell::new(c)))
            .collect())
    }
}

mod rc_refcell_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::{cell::RefCell, rc::Rc};

    pub fn serialize<T, S>(v: &Option<Rc<RefCell<T>>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Clone + Serialize,
        S: Serializer,
    {
        match v {
            Some(rc) => rc.borrow().clone().serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<Rc<RefCell<T>>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let opt: Option<T> = Option::deserialize(deserializer)?;
        Ok(opt.map(|v| Rc::new(RefCell::new(v))))
    }
}
