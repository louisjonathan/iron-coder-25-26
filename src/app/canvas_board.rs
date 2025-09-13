use crate::board::{Board, svg_reader::SvgBoardInfo};
use crate::project::system::Connection;
use egui::{Pos2, Rect, Ui, Sense, Color32, TextureId, Vec2, Id, Response};
use emath::RectTransform;

use std::collections::HashMap;
use egui_extras::{RetainedImage};
use std::vec::Vec;

pub struct CanvasBoard {
	pub board: Board,
	retained_image: RetainedImage,
	texture_id: Option<TextureId>,
	display_size: Vec2,
	image_rect: Rect,
	pin_locations: Vec<(String, Rect)>,
	selected: bool,
	canvas_pos: Vec2,
}

impl CanvasBoard {
    pub fn new(board: &Board) -> Option<Self> {
		if let Some(svg_board_info) = &board.svg_board_info {
			let retained_image = RetainedImage::from_color_image("board_picture", svg_board_info.image.clone());

			let display_size = svg_board_info.physical_size;
			let image_origin = egui::pos2(0.0, 0.0);
			let image_rect = Rect::from_min_size(image_origin, display_size);

			let mut pin_locations = Vec::<(String, Rect)>::new();

			for (pin_name, mut pin_rect) in &svg_board_info.pin_rects {
				// translate the rects so they are in absolute coordinates
				pin_rect = pin_rect.translate(image_rect.left_top().to_vec2());
				pin_locations.push((pin_name.clone(), pin_rect));
			}

			Some(Self {
				board: board.clone(),
				retained_image,
				texture_id: None,
				display_size,
				image_rect,
				pin_locations,
				selected: false,
				canvas_pos: Vec2::new(0.0, 0.0),
			})
		} else {
			None
		}
    }

	pub fn draw(&mut self, ui: &mut egui::Ui, to_screen: &RectTransform, mouse_pos: &Pos2, draw_all_pins: bool) {
		let texture_id = self.texture_id.get_or_insert_with(|| {
			self.retained_image.texture_id(ui.ctx())
		});

		let canvas_rect = self.image_rect.translate(self.canvas_pos);
		let transformed_rect = to_screen.transform_rect(canvas_rect);
		ui.painter().image(
			*texture_id,
			transformed_rect,
			egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
			egui::Color32::WHITE,
		);

		if self.selected {
			self.highlight(ui, to_screen);
		}

		for ((pin_name, pin_rect)) in self.pin_locations.iter() {
			let canvas_pin_rect = (*pin_rect).translate(self.canvas_pos);
			let transformed_pin_rect = to_screen.transform_rect(canvas_pin_rect);
			if draw_all_pins || transformed_pin_rect.contains(*mouse_pos)
			{
				self.draw_pin(ui, pin_name, &transformed_pin_rect);
			}
		}
	}

	pub fn highlight(&self, ui: &mut egui::Ui, to_screen: &RectTransform) {
		let canvas_rect = self.image_rect.translate(self.canvas_pos);
		let transformed_rect = to_screen.transform_rect(canvas_rect);
		ui.painter().rect(
			transformed_rect,
			10,
			Color32::from_rgba_unmultiplied(0, 0, 127, 63),
			egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 63)),
			egui::StrokeKind::Outside
		);
	}

	pub fn draw_pin(&self, ui: &mut egui::Ui, pin_name: &String, pin_rect: &Rect) {
		let pin_name_color = Color32::from_rgba_unmultiplied(0, 255, 0, 63);
		let pin_color = Color32::from_rgba_unmultiplied(0, 255, 0, 191);

		let pin_r = pin_rect.height() / 2.0;

		ui.painter().circle_filled(
			pin_rect.center(),
			pin_r,
			pin_color,
		);

		let text_rect = ui.painter().text(
			pin_rect.center()+Vec2{x:pin_r+1.0,y:0.0},
			egui::Align2::LEFT_CENTER,
			format!("{}", &pin_name),
			egui::FontId::monospace(pin_r*2.0),
			pin_name_color,
		);
	}

	pub fn board_interact(&mut self, to_screen: &RectTransform, zoom: &f32, response: &Response, mouse_pos: &Pos2) -> bool {
		let canvas_rect = self.image_rect.translate(self.canvas_pos);
		let transformed_rect = to_screen.transform_rect(canvas_rect);

		if (transformed_rect.contains(*mouse_pos)) {
			if response.clicked() {
				self.selected = !self.selected;
				return true;
			}
	
			if response.dragged() {
				self.selected = true;
				self.canvas_pos += response.drag_delta() / *zoom;
				return true;
			}
		}

		return false;
	}

	pub fn pin_click(&self, to_screen: &RectTransform, response: &Response, mouse_pos: &Pos2) -> Option<String> {
		if !response.clicked() {
			return None;
		}

		for ((pin_name, pin_rect)) in self.pin_locations.iter() {
			let canvas_pin_rect = (*pin_rect).translate(self.canvas_pos);
			let transformed_pin_rect = to_screen.transform_rect(canvas_pin_rect);
			if transformed_pin_rect.contains(*mouse_pos) {
				return Some(pin_name.clone());
			}
		}
		return None;
	}

	pub fn deselect(&mut self) {
		self.selected = false;
	}

	pub fn get_pin_location(&self, pin_name: &String) -> Option<Pos2> {
		self.pin_locations
			.iter()
			.find(|(name, _rect)| name == pin_name)
			.map(|(_name, rect)| rect.center())
	}
}
