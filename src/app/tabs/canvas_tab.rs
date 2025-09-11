use crate::app::canvas_connection::CanvasConnection;
use crate::app::{canvas_board::CanvasBoard, tabs::base_tab::BaseTab};
use crate::app::SharedState;
use crate::board;
use crate::project::system::Connection;
use eframe::egui::{Pos2, Rect, Response, Sense, Ui, Vec2};

use std::collections::HashMap;
use egui_extras::RetainedImage;

pub struct CanvasTab {
    canvas_zoom: f32,
    canvas_offset: Vec2,
}

impl CanvasTab {
    pub fn new() -> Self {
        Self {
            canvas_zoom: 1.0,
            canvas_offset: Vec2::new(0.0, 0.0),
        }
    }
}

impl BaseTab for CanvasTab {
	fn draw(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
		// grab mouse location
        let mouse_screen = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();

        ui.set_clip_rect(ui.max_rect());

		ui.label(format!("Canvas zoom: {}", self.canvas_zoom));
		ui.label(format!("Canvas offset: {}", self.canvas_offset));
		ui.label(format!("Mouse location: {}", mouse_screen));
		
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

        for canvas_board in &mut state.boards_used {
            canvas_board.draw(ui, &to_screen, &mouse_screen);
            canvas_board.interact(ui, &to_screen, &self.canvas_zoom, response.rect);
        }

        for conn in &mut state.connections {
            conn.draw(ui, &to_screen);
        }

        // adjust offset when dragged
        if response.dragged() {
            self.canvas_offset += response.drag_delta();
        }

        if ui.input(|i| i.pointer.secondary_clicked()) {
            state.connections.push(CanvasConnection::new());
        }

        if response.clicked() {
            for canvas_board in &mut state.boards_used {
                canvas_board.deselect();
            }


            if let Some(conn) = state.connections.last_mut() {
                conn.add_point(mouse_screen);
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}