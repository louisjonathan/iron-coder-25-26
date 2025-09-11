use crate::app::tabs::base_tab::BaseTab;
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
		// grab mouse location
        let mouse_screen = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();

        ui.set_clip_rect(ui.max_rect());

		ui.label(format!("Canvas zoom: {}", self.canvas_zoom));
		ui.label(format!("Canvas offset: {}", self.canvas_offset));
		ui.label(format!("Mouse location: {}", mouse_screen));
		
        let response = ui.allocate_response(ui.available_size(), Sense::drag());

		// adjust offset when dragged
        if response.dragged() {
			self.canvas_offset += response.drag_delta() * self.canvas_zoom;
        }

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