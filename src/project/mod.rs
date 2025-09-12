//! Title: Iron Coder Project Module - Module
//! Description: This module contains the Project struct and its associated functionality.
#![allow(warnings)]
use log::{debug, info, warn};

// use std::error::Error;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};

#[cfg(target_arch = "wasm32")]
use opfs::{
    persistent::{self, app_specific_dir, DirectoryHandle, FileHandle, WritableFileStream},
    CreateWritableOptions, GetDirectoryHandleOptions, GetFileHandleOptions,
};
#[cfg(target_arch = "wasm32")]
use opfs::{DirectoryHandle as _, FileHandle as _, WritableFileStream as _};
#[cfg(target_arch = "wasm32")]
use rfd::AsyncFileDialog;
#[cfg(target_arch = "wasm32")]
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use zip::ZipArchive;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsValue;

#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use serde::{Deserialize, Serialize};

use crate::board::Board;
// use crate::app::code_editor::CodeEditor;

pub use system::Connection;

pub mod display;
use display::ProjectViewType;
use std::string::FromUtf8Error;

pub mod egui_helpers;

pub mod system;

use system::System;

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
    WasmError(String),
    FromUtf8(FromUtf8Error),
    TomlDe(toml::de::Error),
}
#[cfg(target_arch = "wasm32")]
impl From<JsValue> for ProjectIOError {
    fn from(e: JsValue) -> Self {
        Self::WasmError(format!("{:?}", e))
    }
}

impl From<FromUtf8Error> for ProjectIOError {
    fn from(e: FromUtf8Error) -> Self {
        Self::FromUtf8(e)
    }
}


impl From<toml::de::Error> for ProjectIOError {
    fn from(e: toml::de::Error) -> Self {
        Self::TomlDe(e)
    }
}
/// A Project represents the highest level of Iron Coder, which contains
/// a main, programmable development board, a set of peripheral development boards,
/// and the project/source code directory
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Project {
    name: String,
    #[cfg( not (target_arch = "wasm32"))]
    location: Option<PathBuf>,
    pub system: System,
    #[serde(skip)]
    #[cfg(target_arch = "wasm32")]
    location: Option<DirectoryHandle>,
    // pub code_editor: CodeEditor,
    // #[serde(skip)]
    terminal_buffer: String,
    #[serde(skip)]
    receiver: Option<std::sync::mpsc::Receiver<String>>,
    current_view: ProjectViewType,
    #[serde(skip)]
    pub known_boards: Vec<Board>,
    // #[serde(skip)]
    // repo: Option<Repository>,
}

// backend functionality for Project struct
#[cfg( not (target_arch = "wasm32"))]
impl Project {
    // Helper function for printing both to logs and to built-in terminal
    fn info_logger(&mut self, msg: &str) {
        info!("{}", msg);
        let msg = msg.to_owned() + "\n";
        self.terminal_buffer += &msg;
    }

    pub fn borrow_name(&mut self) -> &mut String {
        return &mut self.name;
    }

    pub fn has_main_board(&self) -> bool {
        if let Some(_) = self.system.main_board {
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

    pub fn add_board(&mut self, board: Board) {
        match board.is_main_board() {
            true => {
                if self.has_main_board() {
                    info!("project already contains a main board! aborting.");
                    return;
                } else {
                    self.system.main_board = Some(board.clone());
                }
            }
            false => {
                // don't duplicate a board
                if self.system.peripheral_boards.contains(&board) {
                    info!(
                        "project <{}> already contains board <{:?}>",
                        self.name, board
                    );
                    self.terminal_buffer += "project already contains that board\n";
                    return;
                } else {
                    self.system.peripheral_boards.push(board.clone());
                }
            }
        }
    }

    /// Populate the project board list via the app-wide 'known boards' list
    fn load_board_resources(&mut self) {
        info!("updating project boards from known boards list.");
        for b in self.system.get_all_boards_mut().iter_mut() {
            // returns true if the current, project board is equal to the current known_board
            let predicate = |known_board: &&Board| {
                return known_board == b;
            };
            if let Some(known_board) = self.known_boards.iter().find(predicate) {
                **b = known_board.clone();
            } else {
                warn!("Could not find the project board in the known boards list. Was the project manifest \
                       generated with an older version of Iron Coder?")
            }
        }
    }

    /// This method will reload the project based on the current project location
    pub fn reload(&mut self) -> Result {
        if let Some(location) = self.location.clone() {
            self.load_from(&location)
        } else {
            Err(ProjectIOError::NoProjectDirectory)
        }
    }

    /// Load a project from a specified directory, and sync the board assets.
    fn load_from(&mut self, project_directory: &Path) -> Result {
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
            Err(_e) => return Err(ProjectIOError::LoadToTomlError),
        };
        // Now load in certain fields without overwriting others:
        // self.code_editor.close_all_tabs();
        self.name = p.name;
        self.location = Some(project_directory.to_path_buf());
        self.system = p.system;
        self.current_view = p.current_view;
        // sync the assets with the global ones
        self.load_board_resources();
        self.terminal_buffer.clear();
        // Open the repo in the project directory
        // self.repo = match Repository::open(self.get_location()) {
        //     Ok(repo) => Some(repo),
        //     Err(e) => {
        //         info!("Could not open repo: {}", e);
        //         None
        //     },
        // };

        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    async fn load_from(&mut self, project_directory: &DirectoryHandle) -> Result {
        let options = GetFileHandleOptions{create : false};
        let mut project_file = project_directory.get_file_handle_with_options(PROJECT_FILE_NAME, &options).await?;
        let toml_data = match project_file.read().await{
            Ok(ret) => ret,
            Err(e) => return Err(ProjectIOError::FilesystemError)
        };

        let toml_str = match String::from_utf8(toml_data){
            Ok(s) => s,
            Err(e) => {
                warn!("error reading project file: {:?}", e);
                return Err(ProjectIOError::FilesystemError);
            }
        };
        let p: Project = match toml::from_str(&toml_str) {
            Ok(p) => p,
            Err(_e) => return Err(ProjectIOError::LoadToTomlError),
        };
        // Now load in certain fields without overwriting others:
        // self.code_editor.close_all_tabs();
        self.name = p.name;
        self.location = Some(project_directory.into());
        self.system = p.system;
        self.current_view = p.current_view;
        // sync the assets with the global ones
        self.load_board_resources();
        self.terminal_buffer.clear();
        // Open the repo in the project directory
        // self.repo = match Repository::open(self.get_location()) {
        //     Ok(repo) => Some(repo),
        //     Err(e) => {
        //         info!("Could not open repo: {}", e);
        //         None
        //     },
        // };

        Ok(())
    }

    /// Prompt the user to select project directory to open
    pub fn open(&mut self) -> Result {
        if let Some(project_directory) = FileDialog::new().pick_folder() {
            self.load_from(&project_directory)
        } else {
            info!("project open aborted");
            Err(ProjectIOError::FilePickerAborted)
        }
    }

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
                    self.terminal_buffer += "beware of overwriting and existing project file!\n";
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

    // // Build the code with Cargo
    // fn build(&mut self, ctx: &egui::Context) {
    //     // Make sure we have a valid path
    //     if let Some(path) = &self.location {
    //         info!("building project at {}", path.display().to_string());
    //         // self.code_editor.save_all().unwrap_or_else(|_| warn!("error saving tabs!"));
    //         let cmd = duct::cmd!("cargo", "-Z", "unstable-options", "-C", path.as_path().to_str().unwrap(), "build");
    //         self.run_background_commands(&[cmd], ctx);
    //     } else {
    //         self.info_logger("project needs a valid working directory before building");
    //     }
    // }

    // // Load the code (for now using 'cargo run')
    // fn load_to_board(&mut self, ctx: &egui::Context) {
    //     if let Some(path) = &self.location {
    //         let cmd = duct::cmd!("cargo", "-Z", "unstable-options", "-C", path.as_path().to_str().unwrap(), "run");
    //         self.run_background_commands(&[cmd], ctx);
    //         self.info_logger("Successfully flashed board.");
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

    // Generate the Cargo project template based on the main board template (if it has one).
    // The template will be written to the project directory.
    // TODO - generally more useful error returns, i.e. if the cargo generate command returns a
    // non-zero exit status, or if the project directory already contains a Cargo project.
    // pub fn generate_cargo_template(&mut self, ctx: &egui::Context) -> Result {
    //     info!("generating project template");
    //     let mut cmds: Vec<duct::Expression> = vec![];
    //     if let Some(mb) = &self.system.main_board {
    //         if let Some(template_dir) = mb.get_template_dir() {
    //             let cmd = duct::cmd!(
    //                 "cargo",
    //                 "generate",
    //                 "--path",
    //                 template_dir.as_path().to_str().unwrap(),
    //                 "--name",
    //                 self.name.clone(),
    //                 "--destination",
    //                 self.get_location(),
    //                 "--init",
    //             );
    //             cmds.push(cmd);
    //         } else {
    //             return Err(ProjectIOError::NoProjectTemplate);
    //         }
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
}


#[cfg(target_arch = "wasm32")]
impl Project {
    // Helper function for printing both to logs and to built-in terminal
    fn info_logger(&mut self, msg: &str) {
        info!("{}", msg);
        let msg = msg.to_owned() + "\n";
        self.terminal_buffer += &msg;
    }

    pub fn borrow_name(&mut self) -> &mut String {
        return &mut self.name;
    }

    pub fn has_main_board(&self) -> bool {
        if let Some(_) = self.system.main_board {
            return true;
        } else {
            return false;
        }
    }
    
    pub fn get_location(&self) -> String {
        return String::from("");
    }

    pub fn add_board(&mut self, board: Board) {
        match board.is_main_board() {
            true => {
                if self.has_main_board() {
                    info!("project already contains a main board! aborting.");
                    return;
                } else {
                    self.system.main_board = Some(board.clone());
                }
            }
            false => {
                // don't duplicate a board
                if self.system.peripheral_boards.contains(&board) {
                    info!(
                        "project <{}> already contains board <{:?}>",
                        self.name, board
                    );
                    self.terminal_buffer += "project already contains that board\n";
                    return;
                } else {
                    self.system.peripheral_boards.push(board.clone());
                }
            }
        }
    }

    /// Populate the project board list via the app-wide 'known boards' list
    fn load_board_resources(&mut self) {
        info!("updating project boards from known boards list.");
        for b in self.system.get_all_boards_mut().iter_mut() {
            // returns true if the current, project board is equal to the current known_board
            let predicate = |known_board: &&Board| {
                return known_board == b;
            };
            if let Some(known_board) = self.known_boards.iter().find(predicate) {
                **b = known_board.clone();
            } else {
                warn!("Could not find the project board in the known boards list. Was the project manifest \
                       generated with an older version of Iron Coder?")
            }
        }
    }

    /// This method will reload the project based on the current project location
    pub fn reload( project_rc : Rc<RefCell<Self>>) -> Result {
        spawn_local(async move {
            let project = Rc::clone(&project_rc);
            let res : Result = (async {
                let location = match project.borrow().location.clone() {
                    Some(res) => res,
                    None => return Err(ProjectIOError::LoadToTomlError),
                };
                Project::load_from(project,location);
                Ok(())
            }).await;
            if let Err(e) = res {
                eprintln!("Error during reload: {:?}",e);
            }
        });
        Ok(())
    }
    pub fn load_from(project_rc: Rc<RefCell<Self>>, project_directory: DirectoryHandle) -> Result {
        spawn_local(async move {
            let project = Rc::clone(&project_rc);
            let res : Result = (async {
                let mut project_mut = project.borrow_mut();
                let options = GetFileHandleOptions{create : false};
                let mut project_file = project_directory.get_file_handle_with_options(PROJECT_FILE_NAME, &options).await?;
                let toml_data = project_file.read().await?;

                let toml_str =  String::from_utf8(toml_data)?;
                let p: Project = toml::from_str(&toml_str)?;
                project_mut.name = p.name;
                project_mut.location = Some(project_directory);
                project_mut.system = p.system;
                project_mut.current_view = p.current_view;
                
                project_mut.load_board_resources();
                project_mut.terminal_buffer.clear();
                Ok(())
                }).await;
                if let Err(e) = res {
                    eprintln!("Error during project open: {:?}", e);
                }
        });
        
        Ok(())
    }
    
   pub fn open(project_rc: Rc<RefCell<Self>>) {
        spawn_local(async move {
            let res: Result = (async {
                let project = Rc::clone(&project_rc);
    
                if let Some(file_handle) = AsyncFileDialog::new()
                    .add_filter("zip file", &["zip"])
                    .pick_file()
                    .await
                {
                    let file_bytes = file_handle.read().await;
                    
                    Project::read_zip_into_opfs(file_bytes);

                    let root_dir = opfs::persistent::app_specific_dir().await?;
    
                    Project::load_from(project,root_dir);
                }
                Ok(())
            }).await;
            if let Err(e) = res {
                eprintln!("Error during project open: {:?}", e);
            }
        });
    }

    async fn read_zip_into_opfs(file_bytes: Vec<u8>) -> Result {
        let cursor = Cursor::new(file_bytes);

        // 4. Create a ZipArchive from the cursor
        let mut zip_archive = match ZipArchive::new(cursor) {
            Ok(archive) => archive,
            Err(e) => {
                // Handle the error (e.g., the file is not a valid zip)
                info!("Failed to open zip archive: {:?}", e);
                return Ok(());
            }
        };

        // 5. Get a handle to the OPFS root directory
        let root_dir = match opfs::persistent::app_specific_dir().await {
            Ok(ret) => ret,
            Err(e) => return Err(ProjectIOError::FilePickerAborted),
        };

        // 6. Iterate through all files in the zip archive
        for i in 0..zip_archive.len() {
            let mut file_in_zip = match zip_archive.by_index(i) {
                Ok(file) => file,
                Err(e) => {
                    info!("Failed to get file from zip: {:?}", e);
                    continue;
                }
            };

            // 7. Get the file's path within the zip and check if it's a directory
            let outpath = match file_in_zip.enclosed_name() {
                Some(path) => path.to_owned(),
                None => {
                    info!("File in zip has an invalid path.");
                    continue;
                }
            };
            let sanitized_path = outpath
                .to_string_lossy()
                .replace("/", "-")
                .replace("\\", "-");
            let sanitized_path = std::path::Path::new(&sanitized_path);

            // 8. If the item is a directory, create it and continue
            if file_in_zip.is_dir() {
                let dir_opts = GetDirectoryHandleOptions { create: true };
                if let Err(e) = root_dir
                    .get_directory_handle_with_options(sanitized_path.to_str().unwrap(), &dir_opts)
                    .await
                {
                    info!("Failed to create directory {:?}: {:?}", sanitized_path, e);
                }
                continue;
            }

            // 9. If the item is a file, get its handle and write the contents
            let file_opts = GetFileHandleOptions { create: true };
            let mut file_handle = match root_dir
                .get_file_handle_with_options(sanitized_path.to_str().unwrap(), &file_opts)
                .await
            {
                Ok(handle) => handle,
                Err(e) => {
                    info!("Failed to get file handle {:?}: {:?}", sanitized_path, e);
                    continue;
                }
            };

            let writable_opts = CreateWritableOptions {
                keep_existing_data: false,
            };
            let mut file_stream = match file_handle
                .create_writable_with_options(&writable_opts)
                .await
            {
                Ok(stream) => stream,
                Err(e) => {
                    info!(
                        "Failed to create writable stream for {:?}: {:?}",
                        sanitized_path, e
                    );
                    continue;
                }
            };

            // 10. Copy the data from the zip to the new file
            let mut buffer = Vec::new();
            if let Err(e) = std::io::copy(&mut file_in_zip, &mut buffer) {
                info!("Failed to read from zip file: {:?}", e);
                continue;
            }
            if let Err(e) = file_stream.write_at_cursor_pos(buffer).await {
                info!("Failed to write to file {:?}: {:?}", sanitized_path, e);
                continue;
            }

            // 11. Close the stream
            if let Err(e) = file_stream.close().await {
                info!("Failed to close file stream: {:?}", e);
            }
        }
        Ok(())
    }


    // Save files on OPFS end
    pub fn save(&mut self) -> io::Result<()> {
            // let mut project_file = project_directory.get_file_handle_with_options(PROJECT_FILE_NAME, &options).await?;
            // info!(
            //     "saving project file to {}",
            //     project_file.display().to_string()
            // );

            // match toml::to_string(self) {
            //     Ok(contents) => {
            //         fs::write(project_file, contents)?;
            //     }
            //     Err(e) => {
            //         warn!("couldn't save project to toml file!! {:?}", e);
            //     }
            // }
            // self.code_editor.save_all().unwrap_or_else(|_| warn!("error saving tabs!"));
            Ok(())
    }
    
    // Save project to zip
    #[cfg(target_arch = "wasm32")]
    pub fn save_as(&mut self, create_containing_folder: bool) -> io::Result<()> {
        info!("not yet supported!!");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "not yet supported!!!",
        ));
    }


}
