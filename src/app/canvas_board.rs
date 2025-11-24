use crate::app::SharedState;
use crate::app::canvas_connection::CanvasConnection;
use crate::app::connection_wizard::ConnectionWizard;
use crate::board::{Board, svg_reader::SvgBoardInfo};
use egui::{
    Color32, Context, Id, PointerButton, Pos2, Rect, Response, Sense, TextureHandle, TextureId, Ui,
    Vec2,
};
use emath::RectTransform;

use egui_extras::RetainedImage;
use std::collections::HashMap;
use std::ptr::eq;
use std::vec::Vec;

use std::cell::RefCell;
use std::rc::Rc;

use crate::project::Project;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct CanvasBoard {
    pub id: Uuid,
    board_name: String,
    #[serde(skip)]
    pub board: Rc<Board>,
    #[serde(skip)]
    texture_handle: Option<TextureHandle>,
    #[serde(skip)]
    display_size: Vec2,
    #[serde(skip)]
    image_rect: Rect,
    #[serde(skip)]
    pub pin_locations: HashMap<u32, Rect>,
    pub canvas_pos: Vec2,
    #[serde(skip)]
    pub connections: Vec<Rc<RefCell<CanvasConnection>>>,
    connection_ids: Vec<Uuid>,
    #[serde(skip)]
    canvas_rect: Rect,
    #[serde(skip)]
    is_being_dragged: bool,
}

impl Default for CanvasBoard {
    fn default() -> Self {
        Self {
            id: Uuid::default(),
            board_name: String::new(),
            board: Rc::new(Board::default()),
            texture_handle: None,
            display_size: Vec2::ZERO,
            image_rect: Rect::ZERO,
            pin_locations: HashMap::new(),
            canvas_pos: Vec2::ZERO,
            connection_ids: Vec::new(),
            connections: Vec::new(),
            canvas_rect: Rect::ZERO,
            is_being_dragged: false,
        }
    }
}

impl CanvasBoard {
    pub fn new(board: &Rc<Board>) -> Option<Self> {
        if let Some(svg_board_info) = &board.svg_board_info {
            let display_size = svg_board_info.physical_size;
            let image_origin = egui::pos2(0.0, 0.0);
            let image_rect = Rect::from_min_size(image_origin, display_size);

            let mut pin_locations = HashMap::new();

            for (pin_name, pin_rect) in &svg_board_info.pin_rects {
                if let Some(pin_num) = pin_name.parse::<u32>().ok() {
                    // translate the rects so they are in absolute coordinates
                    let pin_rect = &pin_rect.translate(image_rect.left_top().to_vec2());
                    pin_locations.insert(pin_num, pin_rect.clone());
                }
            }

            let canvas_rect = Rect::ZERO;

            let canvas_rect = Rect::ZERO;

            Some(Self {
                id: Uuid::new_v4(),
                board_name: board.name.clone(),
                board: board.clone(),
                texture_handle: None,
                display_size,
                image_rect,
                pin_locations,
                canvas_pos: Vec2::new(0.0, 0.0),
                connections: Vec::new(),
                connection_ids: Vec::new(),
                canvas_rect,
                is_being_dragged: false,
            })
        } else {
            None
        }
    }

    fn init_pins(&mut self) {
        if let Some(svg_board_info) = &self.board.svg_board_info {
            for (pin_name, pin_rect) in &svg_board_info.pin_rects {
                if let Some(pin_num) = pin_name.parse::<u32>().ok() {
                    // translate the rects so they are in absolute coordinates
                    let pin_rect = &pin_rect.translate(self.image_rect.left_top().to_vec2());
                    self.pin_locations.insert(pin_num, pin_rect.clone());
                }
            }
        }
    }

    pub fn init_refs(&mut self, kb: &Vec<Rc<Board>>, p: &Project) {
        if self.board_name.is_empty() {
            return;
        }
        println!("LOOKING FOR BOARD: {}", self.board_name);
        if let Some(kb_board) = kb.iter().find(|b| b.get_name() == self.board_name) {
            self.board = kb_board.clone();
        }

        self.init_pins();

        if let Some(svg_board_info) = &self.board.svg_board_info {
            let display_size = svg_board_info.physical_size;
            let image_origin = egui::pos2(0.0, 0.0);
            self.image_rect = Rect::from_min_size(image_origin, display_size);
        }

        self.connections = self
            .connection_ids
            .iter()
            .filter_map(|c_id| {
                p.connections_iter()
                    .find(|c| c.borrow().id == *c_id)
                    .map(|c| c.clone())
            })
            .collect();
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, to_screen: &RectTransform, mouse_pos: &Pos2) {
        self.canvas_update(to_screen);
        if self.texture_handle.is_none() {
            if let Some(svg_board_info) = &self.board.svg_board_info {
                self.texture_handle = Some(ui.ctx().load_texture(
                    self.board.get_name(),
                    svg_board_info.image.clone(),
                    Default::default(),
                ));
            }
        }

        if let Some(texture) = &self.texture_handle {
            ui.painter().image(
                texture.id(),
                self.canvas_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }

    pub fn draw_pins(
        &mut self,
        ui: &mut egui::Ui,
        to_screen: &RectTransform,
        mouse_pos: &Pos2,
        draw_all_pins: bool,
        mut wizard: Option<&mut ConnectionWizard>,
        colorscheme: &HashMap<String, Color32>,
        pin_highlight_color: Color32,
    ) {
        for ((pin, pin_rect)) in self.pin_locations.iter() {
            let t_rect = self.to_canvas(to_screen, pin_rect);

            // Check if wizard can select this pin
            let (can_select, pin_color) = {
                if let Some(wiz) = wizard.as_mut() {
                    let can_select = wiz.can_select_pin(*pin, &self.board);

                    let color = if can_select {
                        // Pin supports the required role - use full brightness
                        pin_highlight_color
                    } else {
                        // Pin doesn't support the required role - use dimmed color
                        Color32::from_rgba_unmultiplied(
                            pin_highlight_color.r(),
                            pin_highlight_color.g(),
                            pin_highlight_color.b(),
                            150, // Dimmed for invalid
                        )
                    };

                    (can_select, color)
                } else {
                    // When no wizard is active, use pin highlight color for all pins on hover
                    (false, pin_highlight_color)
                }
            };

            // Auto-show acceptable pins when wizard is active
            // Draw pin if: 1) Alt held, 2) Hovering, OR 3) Wizard active AND pin is acceptable
            let should_draw =
                draw_all_pins || t_rect.contains(*mouse_pos) || (wizard.is_some() && can_select);

            if should_draw {
                if let Some(name) = self.board.pinout.get_pin_name(pin) {
                    self.draw_pin(ui, name, &t_rect, pin_color);
                }
            }
        }
    }

    pub fn draw_pin_from_number(&self, pin: &u32, ui: &mut egui::Ui, to_screen: &RectTransform) {
        let pin_color = ui.style().visuals.faint_bg_color;
        if let Some(pin_rect) = self.pin_locations.get(pin) {
            let t_rect = self.to_canvas(to_screen, pin_rect);
            if let Some(name) = self.board.pinout.get_pin_name(pin) {
                self.draw_pin(ui, name, &t_rect, pin_color);
            }
        }
    }

    pub fn highlight(&self, ui: &mut egui::Ui, to_screen: &RectTransform) {
        ui.painter().rect(
            self.canvas_rect,
            10,
            Color32::from_rgba_unmultiplied(0, 0, 127, 63),
            egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 63)),
            egui::StrokeKind::Outside,
        );
    }

    pub fn draw_pin(&self, ui: &mut egui::Ui, pin_name: &str, pin_rect: &Rect, color: Color32) {
        //let pin_name_color = Color32::from_rgba_unmultiplied(0, 255, 0, 255);
        //let pin_color = Color32::from_rgba_unmultiplied(0, 255, 0, 255);
        let pin_name_color = color;
        let pin_color = color;
        let pin_r = pin_rect.height() / 2.0;

        ui.painter()
            .circle_filled(pin_rect.center(), pin_r, pin_color);

        let text_rect = ui.painter().text(
            pin_rect.center()
                + Vec2 {
                    x: pin_r + 2.0,
                    y: 0.0,
                },
            egui::Align2::LEFT_CENTER,
            format!("{}", &pin_name),
            egui::FontId::monospace(pin_r * 2.0),
            pin_name_color,
        );
    }

    pub fn canvas_update(&mut self, to_screen: &RectTransform) {
        self.canvas_rect = self.to_canvas(to_screen, &self.image_rect);
    }

    pub fn contains(&self, to_screen: &RectTransform, mouse_pos: &Pos2) -> bool {
        if (self.canvas_rect.contains(*mouse_pos)) {
            return true;
        }
        return false;
    }

    pub fn interact(
        &mut self,
        to_screen: &RectTransform,
        zoom: &f32,
        response: &Response,
        mouse_pos: &Pos2,
    ) -> bool {
        if response.clicked_elsewhere() {
            self.is_being_dragged = false;
            return false;
        }
        if self.contains(to_screen, mouse_pos) {
            if response.clicked() {
                return true;
            }
        }
        if response.drag_started() {
            if self.contains(to_screen, mouse_pos) {
                if !self.connections.is_empty() {
                    return false;
                }
                self.is_being_dragged = true;
                self.canvas_pos += response.drag_delta() / *zoom;
                let canvas_rect = self.image_rect.translate(self.canvas_pos);

                self.canvas_rect = to_screen.transform_rect(canvas_rect);

                return true;
            }
        }
        if self.is_being_dragged {
            if response.drag_released() {
                self.is_being_dragged = false;
            }
            if !self.connections.is_empty() {
                return false;
            }

            self.canvas_pos += response.drag_delta() / *zoom;
            let canvas_rect = self.image_rect.translate(self.canvas_pos);

            self.canvas_rect = to_screen.transform_rect(canvas_rect);
            return true;
        }
        if response.drag_released() {
            self.is_being_dragged = false;
        }

        return false;
    }

    pub fn pin_click(
        &self,
        to_screen: &RectTransform,
        response: &Response,
        mouse_pos: &Pos2,
        ui: &Ui,
    ) -> Option<(u32, PointerButton)> {
        ui.ctx().input(|i| {
            let pressed_button = if i.pointer.button_pressed(PointerButton::Primary) {
                Some(PointerButton::Primary)
            } else if i.pointer.button_pressed(PointerButton::Secondary) {
                Some(PointerButton::Secondary)
            } else if i.pointer.button_pressed(PointerButton::Middle) {
                Some(PointerButton::Middle)
            } else {
                None
            }?;

            for (pin_name, pin_rect) in self.pin_locations.iter() {
                if self.to_canvas(to_screen, pin_rect).contains(*mouse_pos) {
                    return Some((*pin_name, pressed_button));
                }
            }

            None
        })
    }

    pub fn get_pin_location(&self, pin_num: &u32) -> Option<Pos2> {
        self.pin_locations.get(pin_num).map(|rect| rect.center())
    }

    pub fn get_canvas_position(&self) -> Vec2 {
        return self.canvas_pos;
    }

    pub fn get_canvas_rect(&self) -> Rect {
        self.canvas_rect
    }

    pub fn drop_connection(&mut self, r: &Rc<RefCell<CanvasConnection>>) {
        self.connections.retain(|c| !Rc::ptr_eq(c, r));
        self.connection_ids.retain(|c| *c != r.borrow().id);
    }

    pub fn add_connection(&mut self, r: &Rc<RefCell<CanvasConnection>>) {
        self.connection_ids.push(r.borrow().id);
        self.connections.push(r.clone());
    }

    pub fn draw_pins_from_role(&self, ui: &mut egui::Ui, to_screen: &RectTransform, role: &String) {
        if let Some(pins) = self.board.pinout.get_pins_from_role(&role) {
            for p in pins {
                let rect = self.pin_locations.get(p).unwrap();
                let pin_obj = self.board.get_pin(p).unwrap();
                let pin_str =
                    if let Some(interface) = self.board.pinout.get_interface_from_role(role) {
                        if let Some(alias) = pin_obj.aliases.get(interface) {
                            alias
                        } else {
                            if self.board.is_main_board() {
                                if let Some(alias) = pin_obj.aliases.get(role) {
                                    alias
                                } else {
                                    &pin_obj.silkscreen
                                }
                            } else {
                                &pin_obj.silkscreen
                            }
                        }
                    } else {
                        if self.board.is_main_board() {
                            if let Some(alias) = pin_obj.aliases.get(role) {
                                alias
                            } else {
                                &pin_obj.silkscreen
                            }
                        } else {
                            &pin_obj.silkscreen
                        }
                    };
                self.draw_pin(
                    ui,
                    pin_str,
                    &self.to_canvas(to_screen, rect),
                    Color32::from_rgb(0, 255, 0),
                );
            }
        }
    }

    fn to_canvas(&self, to_screen: &RectTransform, rect: &Rect) -> Rect {
        let rect = (*rect).translate(self.canvas_pos);
        to_screen.transform_rect(rect)
    }
}
