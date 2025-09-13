use crate::app::{canvas_board, canvas_connection::CanvasConnection};
use crate::app::tabs::base_tab::BaseTab;
use crate::app::canvas_board::CanvasBoard;
use crate::app::SharedState;
use crate::board;
use crate::project::system::Connection;
use eframe::egui::{Pos2, Rect, Response, Sense, Ui, Vec2, Color32, Stroke, Key, Align2, FontId};
use emath::RectTransform;

use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;
use egui_extras::RetainedImage;

pub struct CanvasTab {
    canvas_zoom: f32,
    canvas_offset: Vec2,
    connection_in_progress: Option<Rc<RefCell<CanvasConnection>>>,
}

impl CanvasTab {
    pub fn new() -> Self {
        Self {
            canvas_zoom: 5.0,
            canvas_offset: Vec2::new(0.0, 0.0),
            connection_in_progress: None
        }
    }
}

impl BaseTab for CanvasTab {
	fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
		// grab mouse location
        let mouse_screen = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();

        ui.set_clip_rect(ui.max_rect());

		// ui.label(format!("Canvas zoom: {}", self.canvas_zoom));
		// ui.label(format!("Canvas offset: {}", self.canvas_offset));
		// ui.label(format!("Mouse location: {}", mouse_screen));
		
        let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());

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

        let draw_all_pins = ui.input(|i| i.key_down(Key::Tab));

        let quit_connection = ui.input(|i| i.key_down(Key::Escape));
        if quit_connection {
            state.connections.pop();
            self.connection_in_progress = None;
        }

        for canvas_board_rc in &state.boards_used {
            let mut canvas_board = canvas_board_rc.borrow_mut();
            canvas_board.draw(ui, &to_screen, &mouse_screen, draw_all_pins);
        }

        if let Some(mut conn) = self.connection_in_progress.take() {
            let mut clicked_pin: Option<String> = None;
            for canvas_board_rc in &state.boards_used {
                let mut canvas_board = canvas_board_rc.borrow_mut();
                if let Some(pin) = canvas_board.pin_click(&to_screen, &response, &mouse_screen) {
                    clicked_pin = Some(pin.clone());
                    drop(canvas_board);

                    let mut connection = conn.borrow_mut();
                    connection.end(canvas_board_rc.clone(), pin.clone());
                    connection.add_point(mouse_canvas);
                    break;
                }
            }
            
            if clicked_pin.is_none() {
                self.connection_in_progress = Some(conn);

                if response.clicked() {
                    if let Some(conn) = state.connections.last_mut() {
                        let mut connection = conn.borrow_mut();
                        connection.add_point(mouse_canvas);
                    }
                }
            }            
        } else {

            let mut clicked_pin: Option<String> = None;
            let mut ignore_canvas = false;

            for canvas_board_rc in &state.boards_used {
                if clicked_pin.is_none() {
                    let mut canvas_board = canvas_board_rc.borrow_mut();
                    if let Some(pin) = canvas_board.pin_click(&to_screen, &response, &mouse_screen) {
                        clicked_pin = Some(pin.clone());
                        drop(canvas_board);
                        let mut conn = Rc::new(RefCell::new(CanvasConnection::new(canvas_board_rc.clone(), pin)));
                        let mut connection = conn.borrow_mut();
                        connection.add_point(mouse_canvas);
                        drop(connection);
                        self.connection_in_progress = Some(conn.clone());
                        state.connections.push(conn.clone());
                        continue;
                    }
                }

                let mut canvas_board = canvas_board_rc.borrow_mut();
                if clicked_pin.is_none() {
                    if canvas_board.board_interact(&to_screen, &self.canvas_zoom, &response, &mouse_screen) {
                        ignore_canvas = true;
                    }
                }
            }

            if clicked_pin == None && !ignore_canvas {
                if response.dragged() {
                    self.canvas_offset += response.drag_delta();
                }

                if response.clicked() {
                    {
                        for canvas_board_rc in &state.boards_used {
                            let mut canvas_board = canvas_board_rc.borrow_mut();
                            canvas_board.deselect();
                        }
                    }
                }
            }

        }

        for conn in &mut state.connections {
            let connection = conn.borrow();
            connection.draw(ui, &to_screen, mouse_canvas);
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
        let grid_color = Color32::from_rgba_unmultiplied(127, 127, 127, 63);
        let spacing = 10.0;

        let screen_rect = ui.clip_rect();
        let world_rect = to_screen.inverse().transform_rect(screen_rect);

        let x_start = (world_rect.min.x / spacing).floor() as i32;
        let x_end   = (world_rect.max.x / spacing).ceil() as i32;

        let y_start = (world_rect.min.y / spacing).floor() as i32;
        let y_end   = (world_rect.max.y / spacing).ceil() as i32;

        for i in x_start..=x_end {
            let x = i as f32 * spacing;
            let p1 = to_screen.transform_pos(Pos2::new(x, world_rect.min.y));
            let p2 = to_screen.transform_pos(Pos2::new(x, world_rect.max.y));
            ui.painter().line_segment([p1, p2], Stroke::new(1.0, grid_color));
        }

        for j in y_start..=y_end {
            let y = j as f32 * spacing;
            let p1 = to_screen.transform_pos(Pos2::new(world_rect.min.x, y));
            let p2 = to_screen.transform_pos(Pos2::new(world_rect.max.x, y));
            ui.painter().line_segment([p1, p2], Stroke::new(1.0, grid_color));
        }
    }
}