//! This module provides functionality for development boards
#![allow(warnings)]
use log::{debug, info, warn};
use uuid::Uuid;

use std::cmp;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::vec::Vec;

use serde::{Deserialize, Serialize};

use ra_ap_ide;

pub mod svg_reader;
use svg_reader::SvgBoardInfo;

pub mod display;

pub mod pinout;
pub use pinout::{Pin, Pinout, GPIODirection};

use std::cell::RefCell;
use std::rc::Rc;

use crate::board::pinout::RoleAssignment;

/// These are the various standard development board form factors
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BoardStandards {
    Feather,
    Arduino,
    RaspberryPi,
    ThingPlus,
    MicroMod,
    ESP32,
}

impl fmt::Display for BoardStandards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoardStandards::Feather => write!(f, "Feather"),
            BoardStandards::Arduino => write!(f, "Arduino"),
            BoardStandards::RaspberryPi => write!(f, "RaspberryPi"),
            BoardStandards::ThingPlus => write!(f, "ThingPlus"),
            BoardStandards::MicroMod => write!(f, "MicroMod"),
            BoardStandards::ESP32 => write!(f, "ESP32"),
            _ => write!(f, "Unknown Dev Board Standard"),
        }
    }
}

/// The board struct defines a board type
#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct Board {
    /// The name of the board
    pub name: String,
    /// The board manufacturer
    manufacturer: String,
    /// Whether or not the board has a processor that can run code
    is_main_board: bool,
    /// A possible form factor that the board adheres to
    standard: Option<BoardStandards>,
    pub cpu: Option<String>,
    ram: Option<isize>,
    flash: Option<isize>,
    /// A list of the interfaces available on the board
    pub pinout: Pinout,
    /// A list of the Syntax Nodes of the BSP calculated by Rust Analyzer
    #[serde(skip)]
    pub ra_values: Vec<ra_ap_ide::StructureNode>,
    /// A list of examples
    #[serde(skip)]
    examples: Vec<PathBuf>,
    /// An local path of a project template
    #[serde(skip)]
    template_dir: Option<PathBuf>,
    /// Possible image loaded from an SVG file, along with size info and pin locations
    #[serde(skip)]
    pub svg_board_info: Option<SvgBoardInfo>,
    /// A list of required crates
    required_crates: Option<Vec<String>>,
    /// A list of related, optional crates
    related_crates: Option<Vec<String>>,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Board {}\n", self.name)?;
        write!(f, "  is main board? {}\n", self.is_main_board)?;
        write!(f, "  num examples: {}\n", self.examples.len())?;
        write!(
            f,
            "  num required crates: {}\n",
            self.required_crates.clone().unwrap_or_default().len()
        )?;
        write!(
            f,
            "  num related crates: {}\n",
            self.related_crates.clone().unwrap_or_default().len()
        )?;
        write!(f, "  has svg info: {}\n", self.svg_board_info.is_some())?;
        write!(f, "  has template: {}\n", self.template_dir.is_some())?;
        Ok(())
    }
}

/// Boards are uniquely identified by their name, and thus comparable.
impl cmp::PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl cmp::Eq for Board {}

/// Boards are uniquely identified by their name, and thus hashable.
impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

/// Basic implementation, including loading boards from the filesystem, and retrieving certain
/// information about them.
impl Board {
    /// Loads a board from its toml description
    fn load_from_toml(path: &Path) -> std::io::Result<Self> {
        let toml_str = fs::read_to_string(path)?;
        let mut b: Board =
            toml::from_str(&toml_str).map_err(|e| std::io::Error::other(e.to_string()))?;

        // See if there is an image
        if let Ok(pic_path) = path.with_extension("svg").canonicalize() {
            // BASED ON SVG WORK
            match SvgBoardInfo::from_path(&pic_path) {
                Ok(svg_board_info) => {
                    info!(
                        "successfully decoded SVG for board {}. Board has physical size: {:?}",
                        b.get_name(),
                        svg_board_info.physical_size
                    );
                    b.svg_board_info = Some(svg_board_info);
                }
                Err(e) => {
                    warn!("error with svg parsing! {:?}", e);
                    return Err(std::io::Error::other("unable to parse board SVG file."));
                }
            };
        } else {
            warn!("no svg file for board {}", b.get_name());
            return Err(std::io::Error::other("no SVG file for board."));
        }

        // See if there are any examples
        if let Ok(examples_path) = path.parent().unwrap().join("examples").canonicalize() {
            for (_i, e) in examples_path.read_dir().unwrap().enumerate() {
                let example_path = e.unwrap().path();
                b.examples.push(example_path);
            }
        }

        return Ok(b);
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn required_crates(&self) -> Option<Vec<String>> {
        self.required_crates.clone()
    }

    pub fn related_crates(&self) -> Option<Vec<String>> {
        self.related_crates.clone()
    }

    pub fn is_main_board(&self) -> bool {
        self.is_main_board
    }

    pub fn get_template_dir(&self) -> Option<PathBuf> {
        return self.template_dir.clone();
    }

    pub fn get_board_standard(&self) -> Option<BoardStandards> {
        self.standard.clone()
    }

    pub fn get_pin(&self, physical: &u32) -> Option<&Pin> {
        self.pinout.pins.iter().find(|p| p.physical == *physical)
    }

    pub fn get_peripheral_pin_interface(&self, physical: &u32) -> Option<&RoleAssignment> {
        if self.is_main_board() {
            return None;
        }
        self.pinout.get_peripheral_pin_interface(physical)
    }
}

/// Iteratively gather the Boards from the filesystem.
pub fn get_boards(boards_dir: &Path) -> Vec<Rc<Board>> {
    let mut r = Vec::<Rc<Board>>::new();
    if let Ok(manufacturers) = fs::read_dir(boards_dir) {
        // first tier of organization is by manufacturer
        for manufacturer in manufacturers {
            let manufacturer = manufacturer.expect("error with manufacturer directory");
            if manufacturer
                .file_type()
                .expect("error parsing file type")
                .is_file()
            {
                continue;
            }
            let boards = fs::read_dir(manufacturer.path())
                .expect("error iterating over files in manufacturer directory");
            for board in boards {
                let board = board.expect("error with Board directory");
                if board
                    .file_type()
                    .expect("error parsing file type within board dir")
                    .is_file()
                {
                    continue;
                }
                let files = fs::read_dir(board.path())
                    .expect("error iterating over files in board directory");
                for file in files {
                    let file = file.expect("error reading file within board directory");
                    if file.path().extension().unwrap_or_default() == "toml" {
                        match Board::load_from_toml(&file.path()) {
                            Ok(mut board) => {
                                let parent = file.path().parent().unwrap().canonicalize().unwrap();
                                // look for a template directory
                                let template_dir = parent.join("template");
                                if let Ok(true) = template_dir.try_exists() {
                                    warn!(
                                        "found template dir for board <{}> at {:?}",
                                        board.name.clone(),
                                        file.path()
                                            .parent()
                                            .unwrap()
                                            .canonicalize()
                                            .unwrap()
                                            .join("template")
                                    );
                                    board.template_dir = Some(template_dir);
                                } else {
                                    warn!(
                                        "no template directory found for board <{}>",
                                        board.name.clone()
                                    );
                                }
                                board.pinout.populate_pins(board.is_main_board());
                                r.push(Rc::new(board));
                            }
                            Err(e) => {
                                println!(
                                    "error loading board from {}: {:?}",
                                    file.path().display().to_string(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    return r;
}
