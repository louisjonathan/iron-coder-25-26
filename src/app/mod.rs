#![allow(warnings)]
pub mod colorschemes;
pub mod icons;
pub mod keybinding;

// This is the example from https://github.com/Adanos020/egui_dock/blob/main/examples/hello.rs
// Modified for the purposes of Iron Coder https://github.com/shulltronics/iron-coder

use std::{default, str::FromStr};

use crate::board::Board;

use std::path::Path;

use eframe::egui::{Pos2, Rect, Response, Sense, Ui, Vec2};

#[cfg(target_arch = "wasm32")]
use eframe::egui;
#[cfg(not(target_arch = "wasm32"))]
use eframe::{egui, NativeOptions};

use egui::text::LayoutJob;
use egui::{frame, Area, Color32, ScrollArea};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};

use emath::{self};
use log::info;

use crate::board;
use crate::project::Project;

use std::collections::HashMap;

use eframe::egui::Key;
use serde::{Deserialize, Serialize};

use crate::app::keybinding::{Keybinding, Keybindings};

use egui_extras::RetainedImage;

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use std::fs;
use std::io::{BufReader, Read, Seek, Write};
use std::path::PathBuf;

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::fs::File;
use strip_ansi_escapes;

// use syntect::easy::HighlightLines;
// use syntect::parsing::SyntaxSet;
// use syntect::highlighting::{ThemeSet, FontStyle};
// use syntect::util::LinesWithEndings;

use crate::project::system::Connection;

static OPENABLE_TABS: &'static [&'static str] = &[
    "Settings",
    "Canvas",
    "File Explorer",
    "Terminal",
    "Board Info",
];

trait BaseTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.label("Default");
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

struct CanvasTab {
    canvas_zoom: f32,
    canvas_offset: Vec2,
}

impl CanvasTab {
    fn new() -> Self {
        Self {
            canvas_zoom: 1.0,
            canvas_offset: Vec2::new(0.0, 0.0),
        }
    }

    fn draw_connection(
        ui: &mut egui::Ui,
        src_pos: egui::Pos2,
        dst_pos: egui::Pos2,
        color: egui::Color32,
    ) -> Response {
        let mut response = ui.allocate_rect(
            egui::Rect::from_points(&[src_pos, dst_pos]),
            egui::Sense::click(),
        );
        // these are public fields, but not exposed in egui documentation!
        // response.hovered = false;
        // response.clicked = false;

        let mut connection_stroke = egui::Stroke { width: 2.0, color };

        let mid_x = src_pos.x + (dst_pos.x - src_pos.x) / 2.0;
        // let mid_y = src_pos.y + (dst_pos.y - src_pos.y) / 2.0;
        // let mid_pos1 = egui::Pos2::new(mid_x, src_pos.y);
        // let mid_pos2 = egui::Pos2::new(mid_x, dst_pos.y);

        let control_scale = ((dst_pos.x - src_pos.x) / 2.0).max(30.0);
        let src_control = src_pos + egui::Vec2::X * control_scale;
        let dst_control = dst_pos - egui::Vec2::X * control_scale;

        let mut line = egui::epaint::CubicBezierShape::from_points_stroke(
            [src_pos, src_control, dst_control, dst_pos],
            false,
            egui::Color32::TRANSPARENT,
            connection_stroke,
        );
        // let mut line = egui::epaint::PathShape::line(
        //     Vec::from([src_pos, mid_pos1, mid_pos2, dst_pos]),
        //     connection_stroke,
        // );

        // construct the painter *before* changing the response rectangle. In fact, expand the rect a bit
        // to avoid clipping the curve. This is done so that the layer order can be changed.
        let mut painter = ui.painter_at(response.rect.expand(10.0));
        let mut layer_id = painter.layer_id();
        layer_id.order = egui::Order::Middle;
        painter.set_layer_id(layer_id);

        if let Some(cursor_pos) = ui.ctx().pointer_interact_pos() {
            // the TOL here determines the spacing of the segments that this line is broken into
            // it was determined experimentally, and used in conjunction with THRESH helps to detect
            // if we are hovering over the line.
            const TOL: f32 = 0.01;
            const THRESH: f32 = 12.0;
            line.for_each_flattened_with_t(TOL, &mut |pos, _| {
                if pos.distance(cursor_pos) < THRESH {
                    // response.hovered = true;
                    // using any_click allows clicks, context menu, etc to be handled.
                    if ui.ctx().input(|i| i.pointer.any_click()) == true {
                        // response.clicked = true;
                    }
                    response.rect =
                        egui::Rect::from_center_size(cursor_pos, egui::Vec2::new(THRESH, THRESH));
                }
            });
        }

        if response.hovered() {
            connection_stroke.color = connection_stroke.color.gamma_multiply(0.5);
            line = egui::epaint::CubicBezierShape::from_points_stroke(
                [src_pos, src_control, dst_control, dst_pos],
                false,
                egui::Color32::TRANSPARENT,
                connection_stroke,
            );
            // line = egui::epaint::PathShape::line(
            //     Vec::from([src_pos, mid_pos1, mid_pos2, dst_pos]),
            //     connection_stroke,
            // );
        }

        // painter.add(bezier);
        painter.add(line);

        response
    }
}

impl BaseTab for CanvasTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.set_clip_rect(ui.max_rect());

        let response = ui.allocate_response(ui.available_size(), Sense::drag());

        if response.dragged() {
            self.canvas_offset += response.drag_delta();
        }

        if response.hovered() {
            let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            let zoom_factor = 1.01;

            if scroll_delta != 0.0 {
                let zoom = if scroll_delta > 0.0 {
                    zoom_factor
                } else {
                    1.0 / zoom_factor
                };

                let mouse_screen = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();

                let rect = response.rect;
                let to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, rect.size() / self.canvas_zoom),
                    rect.translate(self.canvas_offset),
                );

                let mouse_canvas_before = to_screen.inverse().transform_pos(mouse_screen);

                self.canvas_zoom *= zoom;

                let new_to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, rect.size() / self.canvas_zoom),
                    rect.translate(self.canvas_offset),
                );

                let mouse_screen_after = new_to_screen.transform_pos(mouse_canvas_before);

                self.canvas_offset += mouse_screen - mouse_screen_after;
            }
        }

        let rect: Rect = response.rect;
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, rect.size() / self.canvas_zoom),
            rect.translate(self.canvas_offset),
        );

        let mut pin_locations: HashMap<(board::Board, String), egui::Pos2> = HashMap::new();

        for board in state.project.system.get_all_boards().iter_mut() {
            let scale_id = egui::Id::new("system_editor_scale_factor");

            let scale = 5.0;

            let board_id = egui::Id::new(board.get_name());

            let mut pin_clicked: Option<String> = None;

            if let Some(svg_board_info) = board.clone().svg_board_info {
                let retained_image = RetainedImage::from_color_image("pic", svg_board_info.image);

                let texture_id = retained_image.texture_id(ui.ctx());
                let display_size = svg_board_info.physical_size * scale;
                let image_rect = egui::Rect::from_min_size(ui.min_rect().min, display_size);
                let transformed_rect = to_screen.transform_rect(image_rect);

                ui.painter().image(
                    texture_id,
                    transformed_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                let visible_rect = ui.clip_rect();

                for (pin_name, mut pin_rect) in board.clone().svg_board_info.unwrap().pin_rects {
                    // scale the rects the same amount that the board image was scaled
                    pin_rect.min.x *= scale;
                    pin_rect.min.y *= scale;
                    pin_rect.max.x *= scale;
                    pin_rect.max.y *= scale;
                    // translate the rects so they are in absolute coordinates
                    pin_rect = pin_rect.translate(image_rect.left_top().to_vec2());
                    pin_locations.insert((board.clone(), pin_name.clone()), pin_rect.center());

                    let transformed_pin_rect = to_screen.transform_rect(pin_rect);

                    if visible_rect.contains_rect(transformed_pin_rect) {
                        let r = ui.allocate_rect(transformed_pin_rect, egui::Sense::click());
                        if r.clicked() {
                            pin_clicked = Some(pin_name.clone());
                        }
                        if r.hovered() {
                            ui.painter().circle_filled(
                                r.rect.center(),
                                r.rect.height() / 2.0,
                                egui::Color32::GREEN,
                            );
                        }
                        r.clone()
                            .on_hover_text(String::from(board.get_name()) + ":" + &pin_name);
                        r.clone().context_menu(|ui| {
                            ui.label("a pin-level menu option");
                        });

                        // render the pin overlay, and check for clicks/hovers
                        // Check if a connection is in progress by checking the "connection_in_progress" Id from the ctx memory.
                        // This is set to true if the user selects "add connection" from the parent container's context menu.
                        let id = egui::Id::new("connection_in_progress");
                        let mut connection_in_progress = ui
                            .ctx()
                            .data_mut(|data| data.get_temp_mut_or(id, false).clone());

                        if connection_in_progress {
                            ui.ctx().output_mut(|o| {
                                o.cursor_icon = egui::CursorIcon::PointingHand;
                            });
                        }

                        if connection_in_progress && r.clicked() {
                            println!("PRESSED");
                            // check conditions for starting/ending a connection
                            match state.project.system.in_progress_connection_start {
                                None => {
                                    ui.ctx().data_mut(|data| {
                                        data.insert_temp(
                                            egui::Id::new("connection_start_pos"),
                                            r.rect.center(),
                                        );
                                    });
                                    state.project.system.in_progress_connection_start =
                                        Some((board.clone(), pin_name.clone()));
                                }
                                Some((ref start_board, ref start_pin)) => {
                                    // add the connection to the system struct
                                    let c = Connection {
                                        name: format!(
                                            "connection_{}",
                                            state.project.system.connections.len()
                                        ),
                                        start_board: start_board.clone(),
                                        start_pin: start_pin.clone(),
                                        end_board: board.clone(),
                                        end_pin: pin_name.clone(),
                                        interface_mapping: board::pinout::InterfaceMapping::default(
                                        ),
                                    };
                                    state.project.system.connections.push(c);
                                    // clear the in_progress_connection fields
                                    state.project.system.in_progress_connection_start = None;
                                    state.project.system.in_progress_connection_end = None;
                                    // and end the connection.
                                    connection_in_progress = false;
                                    ui.ctx().data_mut(|data| {
                                        data.insert_temp(id, connection_in_progress);
                                        data.remove::<egui::Pos2>(egui::Id::new(
                                            "connection_start_pos",
                                        ));
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut connection_to_remove: Option<Connection> = None;
        for connection in state.project.system.connections.iter_mut() {
            // get the start and end pin locations. If they're not in the map (which they should be...), just skip
            let start_loc: egui::Pos2 = match pin_locations
                .get(&(connection.start_board.clone(), connection.start_pin.clone()))
            {
                Some(sl) => *sl,
                None => continue,
            };
            let end_loc: egui::Pos2 = match pin_locations
                .get(&(connection.end_board.clone(), connection.end_pin.clone()))
            {
                Some(el) => *el,
                None => continue,
            };
            // draw the connection and perform interactions.
            let c = match connection.interface_mapping.interface.iface_type {
                board::pinout::InterfaceType::I2C => egui::Color32::RED,
                board::pinout::InterfaceType::UART => egui::Color32::BLUE,
                board::pinout::InterfaceType::SPI => egui::Color32::YELLOW,
                board::pinout::InterfaceType::NONE => egui::Color32::GREEN,
                _ => egui::Color32::WHITE,
            };
            let resp = CanvasTab::draw_connection(ui, start_loc, end_loc, c);
            // Connection-level right click menu
            resp.context_menu(|ui| {
                ui.label("connection name:");
                ui.text_edit_singleline(&mut connection.name);
                ui.separator();
                ui.label("connection type:");
                for iface_type in enum_iterator::all::<board::pinout::InterfaceType>() {
                    ui.selectable_value(
                        &mut connection.interface_mapping.interface.iface_type,
                        iface_type,
                        format!("{:?}", iface_type),
                    );
                }
                ui.separator();
                if ui.button("delete connection").clicked() {
                    connection_to_remove = Some(connection.clone());
                }
            });
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct FileTab {
    path: Option<PathBuf>,
    code: String,
    file: Option<fs::File>,
    synced: bool,
}

impl FileTab {
    fn default() -> Self {
        Self {
            code: String::new(),
            path: None,
            file: None,
            synced: false,
        }
    }

    fn load_from_file(&mut self, file_path: &Path) -> std::io::Result<()> {
        self.code.clear();
        self.path = Some(file_path.canonicalize()?);
        self.file = Some(
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(file_path)?,
        );
        if let Some(file) = &mut self.file {
            file.read_to_string(&mut self.code)?;
            self.synced = true;
        }
        Ok(())
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(file) = &mut self.file {
            file.rewind()?;
            file.set_len(0)?;
            file.write(self.code.as_bytes())?;
            file.sync_all()?;
            self.synced = true;
        }
        Ok(())
    }
}

impl BaseTab for FileTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
            let former_contents = self.code.clone();
            let resp = ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .font(egui::TextStyle::Name("EditorFont".into()))
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .frame(false),
            );
            // check if the code has changed, so we can set the synced flag
            if self.synced && self.code != former_contents {
                self.synced = false;
            }
            // See if a code snippet was released over the editor.
            // TODO -- if so, insert it on the proper line
            ui.ctx().memory_mut(|mem| {
                let id = egui::Id::new("released_code_snippet");
                let data: Option<String> = mem.data.get_temp(id);
                if let Some(value) = data {
                    if resp.hovered() {
                        info!("found a released code snippet!");
                        mem.data.remove::<String>(id);
                        self.code += &value;
                    }
                }
            });
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct FileExplorerTab {
    root_dir: PathBuf,
    expanded_dirs: HashMap<PathBuf, Vec<PathBuf>>,
}

impl FileExplorerTab {
    fn new() -> Self {
        let root_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            root_dir,
            expanded_dirs: HashMap::new(),
        }
    }

    fn read_dir(dir: &PathBuf) -> Vec<PathBuf> {
        fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .collect()
            })
            .unwrap_or_else(|_| vec![])
    }

    fn toggle_dir(&mut self, dir: PathBuf) {
        if self.expanded_dirs.contains_key(&dir) {
            self.expanded_dirs.remove(&dir);
        } else {
            let contents = Self::read_dir(&dir);
            self.expanded_dirs.insert(dir, contents);
        }
    }
}

impl BaseTab for FileExplorerTab {
    fn draw(&mut self, ui: &mut egui::Ui, _state: &mut SharedState) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            fn draw_directory(
                ui: &mut egui::Ui,
                dir: &PathBuf,
                expanded_dirs: &HashMap<PathBuf, Vec<PathBuf>>,
                toggle_dir: &mut dyn FnMut(PathBuf),
                max_visible: usize,
                visible_count: &mut usize,
                depth: usize,
            ) {
                if *visible_count >= max_visible {
                    return; // hack to avoid lag from having too many items open
                }

                let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();
                ui.horizontal(|ui| {
                    ui.add_space(depth as f32 * 16.0);
                    if ui.button(format!("{}", dir_name)).clicked() {
                        toggle_dir(dir.clone());
                    }
                });

                *visible_count += 1;

                if let Some(contents) = expanded_dirs.get(dir) {
                    for entry in contents {
                        if *visible_count >= max_visible {
                            break;
                        }

                        if entry.is_dir() {
                            draw_directory(
                                ui,
                                entry,
                                expanded_dirs,
                                toggle_dir,
                                max_visible,
                                visible_count,
                                depth + 1,
                            );
                        } else {
                            let file_name = entry.file_name().unwrap_or_default().to_string_lossy();
                            ui.horizontal(|ui| {
                                ui.add_space((depth + 1) as f32 * 16.0);
                                ui.label(format!("{}", file_name));
                            });
                            *visible_count += 1;
                        }
                    }
                }
            }

            let expanded_dirs = self.expanded_dirs.clone();
            let root_dir = self.root_dir.clone();
            let mut toggle_dir = {
                let expanded_dirs = &mut self.expanded_dirs;
                move |dir: PathBuf| {
                    if expanded_dirs.contains_key(&dir) {
                        expanded_dirs.remove(&dir);
                    } else {
                        let contents = Self::read_dir(&dir);
                        expanded_dirs.insert(dir, contents);
                    }
                }
            };

            let max_visible = 500;
            let mut visible_count = 0;
            draw_directory(
                ui,
                &root_dir,
                &expanded_dirs,
                &mut toggle_dir,
                max_visible,
                &mut visible_count,
                0,
            );
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct TerminalTab {
    terminal_output: String,
    command_input: String,
}

impl TerminalTab {
    fn new() -> Self {
        TerminalTab {
            terminal_output: String::new(),
            command_input: String::new(),
        }
    }
}

impl BaseTab for TerminalTab {
    fn draw(&mut self, ui: &mut egui::Ui, _state: &mut SharedState) {
        let tab_rect = ui.max_rect();

        let font_height = ui.text_style_height(&egui::TextStyle::Body);
        let frame_padding = ui.style().spacing.item_spacing.y;
        let commandline_height = font_height + 2.0 * frame_padding;
        let commandline_rect = egui::Rect::from_min_size(
            egui::pos2(tab_rect.left(), tab_rect.bottom() - commandline_height),
            egui::vec2(tab_rect.width(), commandline_height),
        );
        ui.allocate_ui_at_rect(commandline_rect, |ui| {
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.command_input)
                        .desired_width(ui.available_width()),
                );
    
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let command = self.command_input.trim();
                    if !command.is_empty() {
                        match Command::new("powershell") // TODO fix this for other OSes just demo for now
                            .arg("-c")
                            .arg(command)
                            .output()
                        {
                            Ok(output) => {
                                let stdout = String::from_utf8_lossy(&output.stdout)
                                    .trim_end()
                                    .to_string();
                                self.terminal_output
                                    .push_str(&format!("> {}\n{}\n", command, stdout));
                            }
                            Err(err) => {
                                self.terminal_output
                                    .push_str(&format!("> {}\n{}\n", command, err));
                            }
                        }
                        self.command_input.clear();
                    }
                }
            });
        });

        let terminalbuffer_rect = egui::Rect::from_min_max(
            tab_rect.min,
            egui::pos2(tab_rect.right(), commandline_rect.top() - 2.0*frame_padding),
        );

        ui.allocate_ui_at_rect(terminalbuffer_rect, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.terminal_output)
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(15)
                            .lock_focus(true)
                            .interactive(false)
                            .desired_width(ui.available_width()),
                    );
                });
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Serialize, Deserialize, Default)]
struct BoardInfoTab {
    chosen_board_idx: Option<usize>,
}
impl BaseTab for BoardInfoTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.heading("Board Selection");
        let mut board: Option<board::Board> = None;
        let available_width = ui.available_width();
        let mut num_cols = (available_width / 260.0) as usize;
        if num_cols == 0 {
            num_cols = 1;
        }
        egui::containers::scroll_area::ScrollArea::vertical().show(ui, |ui| {
            if ui.button("Generate New Board").clicked() {
                todo!();
            }
            ui.label("or select a board from the list below");
            ui.columns(num_cols, |columns| {
                for (i, b) in state.boards.clone().into_iter().enumerate() {
                    let col = i % num_cols;
                    // When a board is clicked, add it to the new project
                    ///@TODO  BoardSelectorWidget
                    if columns[col]
                        .add(board::display::BoardSelectorWidget(b.clone()))
                        .clicked()
                    {
                        board = Some(b.clone());
                        self.chosen_board_idx = Some(i);
                    }
                }

                let last_col = state.boards_used.len();
            });
        });
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
impl BoardInfoTab {
    fn new() -> Self {
        BoardInfoTab {
            chosen_board_idx: None,
        }
    }
    // /// Populate the project board list via the app-wide 'known boards' list
    // fn load_board_resources(&mut self) {
    //     info!("updating project boards from known boards list.");
    //     for b in state.project.system.get_all_boards_mut().iter_mut() {
    //         // returns true if the current, project board is equal to the current known_board
    //         let predicate = |known_board: &&Board| {
    //             return known_board == b;
    //         };
    //         if let Some(known_board) = self.known_boards.iter().find(predicate) {
    //             **b = known_board.clone();
    //         } else {
    //             warn!("Could not find the project board in the known boards list. Was the project manifest \
    //                    generated with an older version of Iron Coder?")
    //         }
    //     }
    // }
    // /// Display the list of available boards in a window, and return one if it was clicked
}

struct SettingsTab {
    should_random_colorscheme: bool,
    should_example_colorscheme: bool,
}
impl SettingsTab {
    fn new() -> Self {
        SettingsTab {
            should_random_colorscheme: false,
            should_example_colorscheme: false,
        }
    }
}

impl BaseTab for SettingsTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        ui.heading("Settings");
        ui.label("Random setting 1");
        let mut s1_value = 50.0;
        ui.add(egui::Slider::new(&mut s1_value, 0.0..=100.0).text("Slider 1"));
        if ui.button("Set example colorscheme").clicked() {
            self.should_example_colorscheme = true;
        }
        if ui.button("Set random colorscheme").clicked() {
            self.should_random_colorscheme = true;
        }
        if ui.button("Save settings").clicked() {
            println!("Settings saved!");
        }
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

struct WindowContext<'a> {
    tabs: &'a mut HashMap<String, Box<dyn BaseTab>>,
    state: &'a mut SharedState,
}

impl<'a> egui_dock::TabViewer for WindowContext<'a> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        if let Some(tab) = self.tabs.get_mut(tab) {
            tab.draw(ui, self.state);
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

struct SharedState {
    keybindings: Keybindings,
    colorschemes: colorschemes::colorschemes,
    project: Project,
    boards: Vec<board::Board>,
    boards_used: Vec<board::Board>,
}

impl SharedState {
    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        let boards_dir = Path::new("./iron-coder-boards");
        let boards: Vec<board::Board> = board::get_boards(boards_dir);

        let mut project = Project::default();
        project.add_board(boards[0].clone());
        let boards_used = project.system.get_all_boards();
        Self {
            keybindings: Keybindings::new(),
            colorschemes: colorschemes::colorschemes::default(),
            project: project,
            boards: boards,
            boards_used,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        let boards: Vec<board::Board> = vec![board::Board::default()];

        #[cfg(target_arch = "wasm32")]
        let boards: Vec<board::Board> = vec![board::Board::default()];

        let mut project = Project::default();
        project.add_board(boards[0].clone());
        let boards_used = project.system.get_all_boards();
        Self {
            keybindings: Keybindings::new(),
            colorschemes: colorschemes::colorschemes::default(),
            project: project,
            boards: boards,
            boards_used,
        }
    }
}

pub struct MainWindow {
    tree: DockState<String>,
    tabs: HashMap<String, Box<dyn BaseTab>>,
    state: SharedState,
}

impl Default for MainWindow {
    fn default() -> Self {
        let mut tree = DockState::new(vec![
            "Canvas".to_owned(),
            "Editor".to_owned(),
            "Settings".to_owned(),
            "./src/main.rs".to_owned(),
        ]);

        let [a, b] = tree.main_surface_mut().split_left(
            NodeIndex::root(),
            0.3,
            vec!["File Explorer".to_owned()],
        );

        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.7, vec!["Terminal".to_owned()]);

        let mut tabs: HashMap<String, Box<dyn BaseTab>> = HashMap::new();

        tabs.insert("Canvas".to_string(), Box::new(CanvasTab::new()));
        tabs.insert("Settings".to_string(), Box::new(SettingsTab::new()));
        tabs.insert(
            "File Explorer".to_string(),
            Box::new(FileExplorerTab::new()),
        );
        tabs.insert("Terminal".to_string(), Box::new(TerminalTab::new()));

        let mut filetab = FileTab::default();
        let path = Path::new("./src/main.rs");
        filetab.load_from_file(path);
        tabs.insert(path.to_string_lossy().into_owned(), Box::new(filetab));

        Self {
            tree: tree,
            tabs: tabs,
            state: SharedState::default(),
        }
    }
}

impl MainWindow {
    // /// Called once before the first frame.
    // pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    //     // This is also where you can customize the look and feel of egui using
    //     // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

    //     // Load previous app state (if any).
    //     // Note that you must enable the `persistence` feature for this to work.
    //     if let Some(storage) = cc.storage {
    //         return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
    //     }

    //     Default::default()
    // }
    pub fn display_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if self.tabs.contains_key("Settings") {
                //make sure settings tab gets current context
                //need help with this line
                let settings_tab = self
                    .tabs
                    .get_mut("Settings")
                    .unwrap()
                    .as_any_mut()
                    .downcast_mut::<SettingsTab>()
                    .unwrap();
                if settings_tab.should_random_colorscheme == true {
                    let random_choice = &colorschemes::colorschemes::get_random_color_scheme();
                    let colors = colorschemes::colorschemes::get_color_scheme(
                        &mut self.state.colorschemes,
                        random_choice,
                    );
                    self.state
                        .colorschemes
                        .set_color_scheme(&ctx, random_choice);

                    ui.visuals_mut().widgets.noninteractive.fg_stroke.color =
                        colors["extreme_bg_color"];
                    ui.visuals_mut().widgets.active.fg_stroke.color = colors["extreme_bg_color"];
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = colors["extreme_bg_color"];
                    ui.visuals_mut().widgets.open.fg_stroke.color = colors["extreme_bg_color"];

                    settings_tab.should_random_colorscheme = false;
                } else if settings_tab.should_example_colorscheme == true {
                    self.state.colorschemes.set_color_scheme(&ctx, &100);
                    settings_tab.should_example_colorscheme = false;
                }
            }
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
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
                    Box::new(BoardInfoTab {
                        chosen_board_idx: None,
                    }),
                );
            }
            _ => {}
        }
        self.tree.push_to_focused_leaf(tab_name);
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.display_menu(ctx, _frame);

        if self.state.keybindings.is_pressed(ctx, "close_tab") {
            // close tab keybind for Jon... (once I figure out how to reliably find the current tab)
            println!("Close tab bind pressed...");
        }

        // if self.tabs.contains_key("Settings") {
        //     //make sure settings tab gets current context
        //     //need help with this line
        //     let settings_tab = self
        //         .tabs
        //         .get_mut("Settings")
        //         .unwrap()
        //         .as_any_mut()
        //         .downcast_mut::<SettingsTab>()
        //         .unwrap();
        //     if settings_tab.should_random_colorscheme == true {
        //         self.state
        //             .colorschemes
        //             .set_color_scheme(&ctx, &colorschemes::colorschemes::get_random_color_scheme());
        //         settings_tab.should_random_colorscheme = false;
        //     } else if settings_tab.should_example_colorscheme == true {
        //         self.state.colorschemes.set_color_scheme(&ctx, &100);
        //         settings_tab.should_example_colorscheme = false;
        //     }
        // }

        // if self.keybindings.is_pressed(ctx, "test_a") {
        //     println!("Test A!");
        // }

        // if self.keybindings.is_pressed(ctx, "test_b") {
        //     println!("Test B!");
        // }

        let mut context = WindowContext {
            tabs: &mut self.tabs,
            state: &mut self.state,
        };

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show_leaf_collapse_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show(ctx, &mut context);
    }
}

// /// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
// pub struct TemplateApp {
//     // Example stuff:
//     label: String,

//     #[serde(skip)] // This how you opt-out of serialization of a field
//     value: f32,
// }

// impl Default for TemplateApp {
//     fn default() -> Self {
//         Self {
//             // Example stuff:
//             label: "Hello World!".to_owned(),
//             value: 2.7,
//         }
//     }
// }

// impl TemplateApp {
//     /// Called once before the first frame.
//     pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
//         // This is also where you can customize the look and feel of egui using
//         // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

//         // Load previous app state (if any).
//         // Note that you must enable the `persistence` feature for this to work.
//         if let Some(storage) = cc.storage {
//             return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
//         }

//         Default::default()
//     }
// }

// impl eframe::App for TemplateApp {
//     /// Called by the frame work to save state before shutdown.
//     fn save(&mut self, storage: &mut dyn eframe::Storage) {
//         eframe::set_value(storage, eframe::APP_KEY, self);
//     }

//     /// Called each time the UI needs repainting, which may be many times per second.
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
//         // For inspiration and more examples, go to https://emilk.github.io/egui

//         egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
//             // The top panel is often a good place for a menu bar:

//             egui::menu::bar(ui, |ui| {
//                 // NOTE: no File->Quit on web pages!
//                 let is_web = cfg!(target_arch = "wasm32");
//                 if !is_web {
//                     ui.menu_button("File", |ui| {
//                         if ui.button("Quit").clicked() {
//                             ctx.send_viewport_cmd(egui::ViewportCommand::Close);
//                         }
//                     });
//                     ui.add_space(16.0);
//                 }

//                 egui::widgets::global_theme_preference_buttons(ui);
//             });
//         });

//         egui::CentralPanel::default().show(ctx, |ui| {
//             // The central panel the region left after adding TopPanel's and SidePanel's
//             ui.heading("eframe template");

//             ui.horizontal(|ui| {
//                 ui.label("Write something: ");
//                 ui.text_edit_singleline(&mut self.label);
//             });

//             ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
//             if ui.button("Increment").clicked() {
//                 self.value += 1.0;
//             }

//             ui.separator();

//             ui.add(egui::github_link_file!(
//                 "https://github.com/emilk/eframe_template/blob/main/",
//                 "Source code."
//             ));

//             ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
//                 powered_by_egui_and_eframe(ui);
//                 egui::warn_if_debug_build(ui);
//             });
//         });
//     }
// }

// fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
//     ui.horizontal(|ui| {
//         ui.spacing_mut().item_spacing.x = 0.0;
//         ui.label("Powered by ");
//         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
//         ui.label(" and ");
//         ui.hyperlink_to(
//             "eframe",
//             "https://github.com/emilk/egui/tree/master/crates/eframe",
//         );
//         ui.label(".");
//     });
// }
