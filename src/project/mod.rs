//! Title: Iron Coder Project Module - Module
//! Description: This module contains the Project struct and its associated functionality.
#![allow(warnings)]
use log::{debug, info, warn};

// use std::error::Error;
use std::fs::{self, read_dir, DirEntry, read_to_string, write};
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use crate::app::{CanvasBoard, CanvasConnection, SharedState};
use crate::board::{get_boards, BoardStandards};
use crate::board::pinout::InterfaceDirection;

use egui::Context;

#[cfg(target_arch = "wasm32")]
use rfd::AsyncFileDialog;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use serde::{Deserialize, Serialize};

use crate::board::Board;
// use crate::app::code_editor::CodeEditor;

pub mod display;
use display::ProjectViewType;

pub mod egui_helpers;

pub mod system;

pub mod toolchain;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use uuid::Uuid;

use system::System;

use syn::{File, Item, ItemFn, Stmt};
use quote::quote;
use prettyplease;

// use git2::Repository;

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
    // pub system: System,
    current_view: ProjectViewType,
	pub source_files: Vec<PathBuf>,

    #[serde(with = "rc_refcell_option")]
    pub main_board: Option<Rc<RefCell<CanvasBoard>>>,

    #[serde(with = "rc_refcell_vec")]
    pub peripheral_boards: Vec<Rc<RefCell<CanvasBoard>>>,

    #[serde(with = "rc_refcell_vec")]
    pub connections: Vec<Rc<RefCell<CanvasConnection>>>,

    #[serde(skip)]
    pub board_map: HashMap<Uuid, Rc<RefCell<CanvasBoard>>>,
}

// backend functionality for Project struct
impl Project {
    // Helper function for printing both to logs and to built-in terminal
    fn info_logger(&mut self, msg: &str) {
        info!("{}", msg);
        let msg = msg.to_owned() + "\n";
        // self.terminal_buffer += &msg;
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

    pub fn add_board(&mut self, board: &Rc<RefCell<Board>>) {
        match board.borrow().is_main_board() {
            true => {
                if self.has_main_board() {
                    info!("project already contains a main board! aborting.");
                    return;
                } else {
                    if let Some(b) = CanvasBoard::new(&board.borrow()) {
                        let b_ref = Rc::new(RefCell::new(b));
                        self.board_map.insert(b_ref.borrow().id.clone(), b_ref.clone());
                        self.main_board = Some(b_ref);
                    }
                }
            }
            false => {
                if let Some(b) = CanvasBoard::new(&board.borrow()) {
                        let b_ref = Rc::new(RefCell::new(b));
                        self.board_map.insert(b_ref.borrow().id.clone(), b_ref.clone());
                        self.peripheral_boards.push(b_ref);
                }
            }
        }
    }

    /// Populate the project board list via the app-wide 'known boards' list
    fn load_board_resources(&mut self, kb: &Vec<Rc<RefCell<Board>>>) {

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
    }

    /// This method will reload the project based on the current project location
    pub fn reload(&mut self, kb: &Vec<Rc<RefCell<Board>>>) -> Result {
        if let Some(location) = self.location.clone() {
            self.load_from(&location, kb)
        } else {
            Err(ProjectIOError::NoProjectDirectory)
        }
    }

    /// Load a project from a specified directory, and sync the board assets.
    pub fn load_from(&mut self, project_directory: &Path, kb: &Vec<Rc<RefCell<Board>>>) -> Result {
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
        self.current_view = p.current_view;
        self.main_board = p.main_board;
        self.peripheral_boards = p.peripheral_boards;
        self.connections = p.connections;
        self.load_board_resources(kb);
		self.find_source_files();

        Ok(())
    }

    /// Prompt the user to select project directory to open
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(&mut self, kb: &Vec<Rc<RefCell<Board>>>) -> Result {
        if let Some(project_directory) = FileDialog::new().pick_folder() {
            self.load_from(&project_directory, kb)
        } else {
            info!("project open aborted");
            Err(ProjectIOError::FilePickerAborted)
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn open(&mut self) -> Result {
        // if let future = async {
        //     let project_directory = rfd::AsyncFileDialog::new().pick_folder().await;
        //     project_directory.load_from(&project_directory).await;
        // } {
        // } else {
        //     info!("project open aborted");
        //     Err(ProjectIOError::FilePickerAborted)
        // }
        ///TODO: Support opening project using zip file
        info!("not yet supported!!");
        Err(ProjectIOError::FilePickerAborted)
    }

    /// Open a file dialog to select a project folder, and then call the save method
    /// TODO - make file dialog have default directory
    #[cfg(not(target_arch = "wasm32"))]
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
                    // self.terminal_buffer += "beware of overwriting and existing project file!\n";
                    return Ok(());
                }
            }
            self.location = Some(project_folder);
            // TODo: find template directory based on "programmable board" (for now just use board 0) -- No longer relevant?
            // if let Some(template_dir) = self.system.boards[0].get_template_dir() {
            //     // copy_recursive(template_dir, project_dir)
            //     let options = fs_extra::dir::CopyOptions::new();
            //     for entry in std::fs::read_dir(template_dir).unwrap() {
            //         let entry = entry.unwrap().path();
            //         if let Err(e) = fs_extra::copy_items(&[entry.clone()], self.location.clone().unwrap(), &options) {
            //             warn!("couldn't copy template item {:?} to new project folder; {:?}", entry, e);
            //         }
            //     }
            // }
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "project save aborted!",
            ));
        }
        self.save()
    }
    #[cfg(target_arch = "wasm32")]
    pub fn save_as(&mut self, create_containing_folder: bool) -> io::Result<()> {
        info!("not yet supported!!");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "not yet supported!!!",
        ));
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
                }
                Err(e) => {
                    warn!("couldn't save project to toml file!! {:?}", e);
                }
            }

            // self.code_editor.save_all().unwrap_or_else(|_| warn!("error saving tabs!"));
            Ok(())
        }
    }

    // Build the code with Cargo
    // pub fn build(&mut self, ctx: &egui::Context) {
    // // Make sure we have a valid path
    //     if let Some(path) = &self.location {
    //         info!("building project at {}", path.display().to_string());
    //         // self.code_editor.save_all().unwrap_or_else(|_| warn!("error saving tabs!"));
    //         let cmd = duct::cmd!("cargo", "build").dir(path);
    //         self.run_background_commands(&[cmd], ctx);
    //     } else {
    //         self.info_logger("project needs a valid working directory before building");
    //     }
    // }

    // pub fn new_file(&mut self) -> io::Result<()> {
    //     if self.location == None {
    //         self.info_logger("must save project before adding files/directories");
    //         return Ok(());
    //     }
    //     if let Some(pathbuf) = FileDialog::new().set_directory(self.location.clone().unwrap()).save_file() {
    //         fs::File::create_new(pathbuf)?;
    //     } else {
    //         warn!("error getting file path");
    //     }
    //     Ok(())
    // }

    // This method will run a series of command sequentially on a separate
    // thread, sending their output through the channel to the project's terminal buffer
    // TODO - fix bug that calling this command again before a former call's thread is
    //   complete will overwrite the rx channel in the Project object. Possible solution
    //   might be to add a command to a queue to be evaluated.
    // fn run_background_commands(&mut self, cmds: &[duct::Expression], ctx: &egui::Context) {
    //     // create comms channel
    //     let context = ctx.clone();
    //     let commands = cmds.to_owned();
    //     let (tx, rx) = std::sync::mpsc::channel();
    //     self.receiver = Some(rx);
    //     let _ = std::thread::spawn(move || {
    //         for cmd in commands.iter() {
    //             let reader = cmd.stderr_to_stdout().unchecked().reader().unwrap();
    //             let mut lines = std::io::BufReader::new(reader).lines();
    //             while let Some(line) = lines.next() {
    //                 let line = line.unwrap() + "\n";
    //                 debug!("sending line through channel");
    //                 tx.send(line).unwrap();
    //                 context.request_repaint();
    //             }
    //         }
    //         info!("leaving thread");
    //     });
    // }

    pub fn generate_cargo_template(&mut self) -> Result {
        if let Some(mb) = &self.main_board {
            if let Some(template_dir) = mb.borrow().board.get_template_dir() {
                
                // if Path::new(&template_dir).join("Makefile.toml").exists(){
                //     let cmd = duct::cmd!("cargo", "make", "modify-config-toml");
                //     match cmd.dir(&template_dir).run() {
                //         Ok(_) => {},
                //         Err(e) => {
                //             warn!("Failed to run cargo make modify-config-toml: {:?}", e);
                //         }
                //     }
                // }
                
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
                    Ok(output) => {
                        Ok(())
                    }
                    Err(e) => {
                        Err(ProjectIOError::FilesystemError)
                    }
                }
            } else {
                return Err(ProjectIOError::NoProjectTemplate);
            }
        } else {
            return Err(ProjectIOError::NoMainBoard);
        }
    }
    // iterate through BSP paths and add the crates to the project
    // TODO: This needs to be changed, likely an issue with
    // updating the crates in the main toml file. Figure out why!
    /*
    for b in self.system.get_all_boards() {
        if let Some(local_bsp) = b.bsp_path {
            let cmd = duct::cmd!(
                "cargo",
                "-Z",
                "unstable-options",
                "-C",
                self.location.clone().unwrap(),
                "add",
                "--path",
                local_bsp,
            );
            cmds.push(cmd);
        }
    }
    */
    //         self.run_background_commands(&cmds, ctx);
    //     } else {
    //         return Err(ProjectIOError::NoMainBoard);
    //     }
    //     // Create a repo to store code
    //     self.repo = match Repository::init(self.get_location()) {
    //         Ok(repo) => Some(repo),
    //         Err(_e) => return Err(ProjectIOError::NoProjectDirectory),
    //     };

    //     Ok(())
    // }

    // Attempt to load code snippets for the provided crate
    // fn load_snippets(&self, base_dir: &Path, crate_name: String) -> io::Result<String> {
    //     let snippets_dir = base_dir.join(crate_name.clone());
    //     if let Ok(true) = snippets_dir.try_exists() {
    //         for entry in snippets_dir.read_dir().unwrap() {
    //             let entry = entry.unwrap();
    //             let contents = std::fs::read_to_string(entry.path())?;
    //             return Ok(contents);
    //         }
    //     } else {
    //         warn!("couldn't load code snippets for crate {}", crate_name);
    //     }
    //     Ok("".to_string())
    // }

    /// Update terminal output
    // pub fn update_terminal_output(&mut self) {
    //     if let Some(rx) = &self.receiver {
    //         while let Ok(line) = rx.try_recv() {
    //             self.terminal_buffer.push_str(&line);
    //         }
    //     }
    // }

    // pub fn get_terminal_output(&self) -> &str {
    //     &self.terminal_buffer
    // }

    // pub fn clear_terminal_output(&mut self) {
    //     self.terminal_buffer.clear();
    // }

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
        if let Some(mb) = &self.main_board {
            if Rc::ptr_eq(board, mb) {
                self.main_board = None;
                return;
            }
        }
        self.peripheral_boards.retain(|c| !Rc::ptr_eq(c, board));
    }

    pub fn remove_connection(&mut self, connection: &Rc<RefCell<CanvasConnection>>) {
        self.connections.retain(|c| !Rc::ptr_eq(c, connection));
    }

    pub fn boards_iter(&self) -> impl Iterator<Item=&Rc<RefCell<CanvasBoard>>> {
        self.main_board.iter().chain(self.peripheral_boards.iter())
    }

    pub fn boards_iter_rev(&self) -> impl Iterator<Item=&Rc<RefCell<CanvasBoard>>> {
        self.peripheral_boards.iter().rev().chain(self.main_board.iter().rev())
    }

    pub fn connections_iter(&self) -> impl Iterator<Item=&Rc<RefCell<CanvasConnection>>> {
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

	fn insert_pin_into_source(&self, path: &PathBuf, conn: &CanvasConnection) {
		let code = read_to_string(&path).unwrap();
		let mut ast: File = syn::parse_str(&code).unwrap();

		for item in &mut ast.items {
			if let Item::Fn(ItemFn { sig, block, .. }) = item {
				if sig.ident == "main" {
					for (i, stmt_in_block) in block.stmts.iter().enumerate() {
						let stmt_str = quote!(#stmt_in_block).to_string();
						if stmt_str.contains("loop") {
							let new_stmt_str = self.generate_pin_statement(conn);
							if let Ok(new_stmt) = syn::parse_str::<Stmt>(&new_stmt_str) {
								block.stmts.insert(i, new_stmt);
							} else {
								eprintln!("Failed to parse generated statement: {}", new_stmt_str);
							}
							break;
						}
					}
				}
			}
		}
		let new_code = prettyplease::unparse(&ast);
		write(&path, new_code);
	}

	fn generate_pin_statement(&self, conn: &CanvasConnection) -> String {
		let sb_rc = conn.get_start_board();
		let sb = sb_rc.borrow().board.clone();
		let eb_rc = conn.get_end_board().unwrap();
		let eb = eb_rc.borrow().board.clone();
		
		let start_pin_name = conn.get_start_pin();
		let end_pin_name = conn.get_end_pin().unwrap();
		let pin_type = eb.pinout.iter()
			.find(|pinout| pinout.pins.iter().any(|p| p == end_pin_name.as_str()))
			.map(|pinout| pinout.interface.direction.clone());		
		let var_name = format!("{}_to_{}", start_pin_name, end_pin_name);

		match sb.get_board_standard() {
			// Some(BoardStandards::Feather) => {
			// },
			Some(BoardStandards::Arduino) => {
				let pin_type_str = match pin_type {
					Some(InterfaceDirection::Output) => "input",
					Some(InterfaceDirection::Input) => "output",
					_ => "input",
				};
				let mutability = if pin_type_str == "output" {"mut "} else {""};
				format!(
					"let {}pin_{} = pins.{}.into_{}();",
					mutability, var_name, start_pin_name, pin_type_str
				)
			},
			// Some(BoardStandards::RaspberryPi) => {
				
			// },
			// Some(BoardStandards::ThingPlus) => {
				
			// },
			// Some(BoardStandards::MicroMod) => {
				
			// },
			Some(BoardStandards::ESP32) => {
				match pin_type {
					Some(InterfaceDirection::Output) => {
						format!(
							"let mut pin_{} = Output::new(_peripherals.GPIO{}, Level::High, OutputConfig::default());",
							var_name, start_pin_name
						)
					}
					Some(InterfaceDirection::Input) => {
						format!(
							"let pin_{} = Input::new(_peripherals.GPIO{}, InputConfig::default().with_pull(Pull::Up));",
							var_name, start_pin_name
						)
					}
					_ => { "".to_string() }
				}
			},
			_ => "".to_string(),
		}
	}
}

mod rc_refcell_vec {
    use std::{rc::Rc, cell::RefCell};
    use serde::{Serializer, Deserializer, Serialize, Deserialize};

    pub fn serialize<T, S>(v: &Vec<Rc<RefCell<T>>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Clone + Serialize,
        S: Serializer,
    {
        // Create a temporary Vec of plain T for serialization
        let plain: Vec<T> = v.iter().map(|c| c.borrow().clone()).collect();
        plain.serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Vec<Rc<RefCell<T>>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        // Deserialize plain Vec<T>
        let plain: Vec<T> = Vec::deserialize(deserializer)?;
        // Wrap each element in Rc<RefCell>
        Ok(plain.into_iter().map(|c| Rc::new(RefCell::new(c))).collect())
    }
}

mod rc_refcell_option {
    use std::{rc::Rc, cell::RefCell};
    use serde::{Serializer, Deserializer, Serialize, Deserialize};

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