use crate::app::{canvas_board, canvas_connection::CanvasConnection};
use crate::app::tabs::base_tab::BaseTab;
use crate::app::canvas_board::CanvasBoard;
use crate::app::canvas_element::CanvasSelection;
use crate::app::SharedState;
use crate::board;
use eframe::egui::{Pos2, Rect, Response, Sense, Ui, Vec2, Color32, Stroke, Key, Align2, FontId};
use egui::color_picker::color_picker_color32;
use emath::RectTransform;
use syntect::highlighting::Color;

use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;
use egui_extras::RetainedImage;


pub struct CanvasTab {
    canvas_zoom: f32,
    canvas_offset: Vec2,
    connection_in_progress: Option<Rc<RefCell<CanvasConnection>>>,
    selection : Option<CanvasSelection>,
}

impl CanvasTab {
    pub fn new() -> Self {
        Self {
            canvas_zoom: 5.0,
            canvas_offset: Vec2::new(0.0, 0.0),
            connection_in_progress: None,
            selection: None,
        }
    }
}

impl BaseTab for CanvasTab {
    fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
		if state.reset_canvas {
			self.reset_canvas();
			state.reset_canvas = false;
		}

        // grab mouse location
        let mouse_screen = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();

        // ui.label(format!("Canvas zoom: {}", self.canvas_zoom));
        // ui.label(format!("Canvas offset: {}", self.canvas_offset));
        // ui.label(format!("Mouse location: {}", mouse_screen));
        
        let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());

        let rect = response.rect;

        if response.hovered() {
            let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            let zoom_factor = 1.01;

            // handle scrolling to zoom on mouse location using transformations
            if scroll_delta != 0.0 {
                let zoom = if scroll_delta > 0.0 {
                    zoom_factor
                } else {
                    1.0 / zoom_factor
                };

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

                // change offset based on where we zoom
                self.canvas_offset += mouse_screen - mouse_screen_after;
            }
        }

        let rect: Rect = response.rect;
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, rect.size() / self.canvas_zoom),
            rect.translate(self.canvas_offset),
        );

        let mouse_pos = mouse_screen - rect.min;
        let mouse_canvas = to_screen.inverse().transform_pos(Pos2::new(mouse_screen.x, mouse_screen.y));

        /* DEBUG DISPLAY MOUSE & MOUSE CANVAS POSITIONS

        ui.painter().text(
            mouse_screen + Vec2{x: 0.0, y: 0.0},
            Align2::LEFT_TOP,
            format!("Raw Mouse: {}", mouse_pos),
            FontId::monospace(12.0),
            Color32::WHITE
        );

        ui.painter().text(
            mouse_screen + Vec2{x: 0.0, y: 12.0},
            Align2::LEFT_TOP,
            format!("Canvas Mouse: {}", mouse_canvas),
            FontId::monospace(12.0),
            Color32::WHITE
        );
        
        */

        self.draw_grid(ui, &to_screen);

        let draw_all_pins = ui.input(|i| i.modifiers.alt);

        if ui.input(|i| i.key_pressed(Key::Delete)) {
			if let Some(s) = self.selection.take() {
				match s {
					CanvasSelection::Board(board) => {
						self.selection = None;
                        state.project.remove_board(&board);
                    }
                    CanvasSelection::Connection(connection) => {
						self.selection = None;
                        state.project.remove_connection(&connection);
                    }
                }
        	}
        }

        let quit_connection = ui.input(|i| i.key_pressed(Key::Escape));
        if quit_connection {
            if self.connection_in_progress.is_some() {
                self.connection_in_progress = None;
            }
        }

        // BOARDS
        for b in state.project.boards_iter() {
            b.borrow_mut().draw(ui, &to_screen, &mouse_screen);
        }

        // CONNECTIONS
        for c in state.project.connections_iter() {
            c.borrow().draw(ui, &to_screen, mouse_canvas);
        }
        if let Some(c) = &self.connection_in_progress {
            c.borrow().draw(ui, &to_screen, mouse_canvas);
        }

        // PINS
        for b in state.project.boards_iter() {
            b.borrow_mut().draw_pins(ui, &to_screen, &mouse_screen, draw_all_pins);
        }

        // Keybind text
        // TODO: bind to keybinds backend
        let mut offset = 0.0;
        ui.painter().text(rect.min+Vec2{x:0.0,y:offset}, Align2::LEFT_TOP, "Alt: Show all pins", FontId::monospace(12.0), Color32::WHITE);
        offset += 12.0;

        if self.selection.is_some() {
            ui.painter().text(rect.min+Vec2{x:0.0,y:offset}, Align2::LEFT_TOP, "Delete: Remove current selection from canvas", FontId::monospace(12.0), Color32::WHITE);
            offset += 12.0;
        }
        
        if self.connection_in_progress.is_some() {
            ui.painter().text(rect.min+Vec2{x:0.0,y:offset}, Align2::LEFT_TOP, "Escape: Quit current connection", FontId::monospace(12.0), Color32::WHITE);
            offset += 12.0;
        }

        /* interaction flow
            1. check for current connection
            2. check pins for click
            3. check connections for click
            4. check boards for click
            5. drag canvas
        */

        // 1
        if let Some(mut conn) = self.connection_in_progress.take() {
            let mut clicked_pin: Option<String> = None;
            let boards: Vec<_> = state.project.boards_iter().cloned().collect();
            for canvas_board_rc in &boards {
                let pin_opt = {
                    let mut canvas_board = canvas_board_rc.borrow_mut();
                    if !canvas_board.contains(&to_screen, &mouse_screen) {
                        None
                    } else {
                        canvas_board.pin_click(&to_screen, &response, &mouse_screen)
                    }
                };

                if let Some(pin) = pin_opt {
                    clicked_pin = Some(pin.clone());

                    let conn_clone = conn.clone();
                    {
                        let mut connection = conn_clone.borrow_mut();

                        let pin_location_opt = {
                            let canvas_board = canvas_board_rc.borrow();
                            canvas_board.get_pin_location(&pin)
                                .map(|loc| loc + canvas_board.get_canvas_position())
                        };

                        if let Some(pin_location) = pin_location_opt {
                            connection.add_end_point(&mouse_canvas, pin_location);
                        }

                        connection.get_start_board().borrow_mut().connections.push(conn_clone.clone());

                        connection.end(canvas_board_rc.clone(), pin.clone());
                    }

                    {
                        let mut canvas_board = canvas_board_rc.borrow_mut();
                        canvas_board.add_connection(&conn_clone);
                    }

                    state.project.add_connection(&conn_clone);
                    break;
                }
            }
            
            if clicked_pin.is_none() {
                self.connection_in_progress = Some(conn);

                if response.clicked() {
                    if let Some(conn) = &self.connection_in_progress {
                        conn.borrow_mut().add_point(mouse_canvas);
                    }
                }
            }            
        } else {

            let mut clicked_pin: Option<String> = None;
            let mut ignore_canvas = false;

            // 2
            let boards: Vec<_> = state.project.boards_iter().cloned().collect();
            for canvas_board_rc in &boards {
                if clicked_pin.is_none() {
                    let pin_opt = {
                        let mut canvas_board = canvas_board_rc.borrow_mut();
                        canvas_board.pin_click(&to_screen, &response, &mouse_screen)
                    };
                    
                    if let Some(pin) = pin_opt {
                        clicked_pin = Some(pin.clone());
                        if self.check_pin_use(canvas_board_rc, &pin, &state.project.connections) {
                            break;
                        }
                        
                        let mut conn = Rc::new(RefCell::new(CanvasConnection::new(canvas_board_rc.clone(), pin.clone())));
                        {
                            let mut connection = conn.borrow_mut();
                            let canvas_board = canvas_board_rc.borrow();
                            if let Some(pin_location) = canvas_board.get_pin_location(&pin) {
                                connection.add_point(pin_location+canvas_board.get_canvas_position());
                            }
                        }

                        {
                            let mut canvas_board = canvas_board_rc.borrow_mut();
                            canvas_board.add_connection(&conn);
                        }

                        self.connection_in_progress = Some(conn.clone());
                        break;
                    }
                }
            }

            if response.clicked() {
                // 3
                let mut connection_clicked = false;
                for c in state.project.connections_iter() {
                    let connection = c.borrow();
                    if connection.contains(&to_screen, &mouse_screen) {
                        connection_clicked = true;
                        self.selection = Some(CanvasSelection::Connection(c.clone()));
                        ignore_canvas = true;
                        break;
                    }
                }

                // 4 only check boards if we didnt click connection
                if !connection_clicked {
                    for b in state.project.boards_iter_rev() {
                        let board = b.borrow();
                        if board.contains(&to_screen, &mouse_screen) {
                            self.selection = Some(CanvasSelection::Board(b.clone()));
                            ignore_canvas = true;
                            break;
                        }
                    }
                }
            }

            if let Some(s) = self.selection.as_mut() {
                if s.interact(&to_screen, &self.canvas_zoom, &response, &mouse_screen) {
                    ignore_canvas = true;
                }
                s.highlight(ui, &to_screen);
            }

            // 5
            if clicked_pin == None && !ignore_canvas {
                if response.dragged() {
                    self.canvas_offset += response.drag_delta();
                }

                if response.clicked() {
                    self.selection = None;
                }
            }

        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CanvasTab {
    fn draw_grid(&self, ui: &mut egui::Ui, to_screen: &RectTransform) {
        let grid_color = Color32::from_rgba_unmultiplied(42, 42, 42, 255);
        let spacing = 10.0;
        let major_spacing = 50.0;

        let screen_rect = ui.clip_rect();
        let canvas_rect = to_screen.inverse().transform_rect(screen_rect);

        let x_start = (canvas_rect.min.x / spacing).floor() as i32;
        let x_end   = (canvas_rect.max.x / spacing).ceil() as i32;

        let y_start = (canvas_rect.min.y / spacing).floor() as i32;
        let y_end   = (canvas_rect.max.y / spacing).ceil() as i32;

        for i in x_start..=x_end {
            let x = i as f32 * spacing;
            let p1 = to_screen.transform_pos(Pos2::new(x, canvas_rect.min.y));
            let p2 = to_screen.transform_pos(Pos2::new(x, canvas_rect.max.y));
            ui.painter().line_segment([p1, p2], Stroke::new(1.0, grid_color));
        }

        for j in y_start..=y_end {
            let y = j as f32 * spacing;
            let p1 = to_screen.transform_pos(Pos2::new(canvas_rect.min.x, y));
            let p2 = to_screen.transform_pos(Pos2::new(canvas_rect.max.x, y));
            ui.painter().line_segment([p1, p2], Stroke::new(1.0, grid_color));
        }

        let x_major_start = (canvas_rect.min.x / major_spacing).floor() as i32;
        let x_major_end   = (canvas_rect.max.x / major_spacing).ceil() as i32;

        let y_major_start = (canvas_rect.min.y / major_spacing).floor() as i32;
        let y_major_end   = (canvas_rect.max.y / major_spacing).ceil() as i32;

        for i in x_major_start..=x_major_end {
            let x = i as f32 * major_spacing;
            let p1 = to_screen.transform_pos(Pos2::new(x, canvas_rect.min.y));
            let p2 = to_screen.transform_pos(Pos2::new(x, canvas_rect.max.y));
            ui.painter().line_segment([p1, p2], Stroke::new(3.0, grid_color));
        }

        for j in y_major_start..=y_major_end {
            let y = j as f32 * major_spacing;
            let p1 = to_screen.transform_pos(Pos2::new(canvas_rect.min.x, y));
            let p2 = to_screen.transform_pos(Pos2::new(canvas_rect.max.x, y));
            ui.painter().line_segment([p1, p2], Stroke::new(3.0, grid_color));
        }
    }

    fn check_pin_use(&self, board: &Rc<RefCell<CanvasBoard>>, pin_name: &String, connections: &Vec<Rc<RefCell<CanvasConnection>>>) -> bool {
        for c in connections {
            let connection = c.borrow();
            if Rc::ptr_eq(&connection.get_start_board(), board) && pin_name.eq(&connection.get_start_pin()) {
                return true;
            }
            if let Some(eb) = connection.get_end_board() {
                if Rc::ptr_eq(&eb, board) && pin_name.eq(&connection.get_end_pin().unwrap()) {
                    return true;
                }
            }
        }
        return false;
    }

	pub fn fit_to_screen(&mut self, state: &mut SharedState, screen_rect: &Rect) {
		let mut min = Vec2::splat(f32::INFINITY);
    	let mut max = Vec2::splat(f32::NEG_INFINITY);

		for b_ref in state.project.boards_iter() {
			let b = b_ref.borrow();
			let p = b.get_canvas_position();

			min = min.min(p);
			max = max.max(p);
		}
	}

    pub fn reset_canvas(&mut self) {
        self.canvas_zoom = 5.0;
        self.canvas_offset = Vec2::new(0.0, 0.0);
        self.connection_in_progress = None;
        self.selection = None;
    }
}