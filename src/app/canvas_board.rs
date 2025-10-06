use crate::app::canvas_connection::CanvasConnection;
use crate::board::{Board, svg_reader::SvgBoardInfo};
use crate::app::SharedState;
use egui::{Pos2, Rect, Ui, Sense, Color32, TextureId, Vec2, Id, Response, Context, TextureHandle};
use emath::RectTransform;

use std::collections::HashMap;
use std::ptr::eq;
use egui_extras::{RetainedImage};
use std::vec::Vec;

use std::rc::Rc;
use std::cell::RefCell;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::project::Project;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CanvasBoard {
	pub id: Uuid,
	pub board: Board,
	#[serde(skip)]
	texture_handle: Option<TextureHandle>,
	display_size: Vec2,
	#[serde(skip)]
	image_rect: Rect,
	pin_locations: Vec<(String, Rect)>,
	canvas_pos: Vec2,
	#[serde(skip)]
	pub connections: Vec<Rc<RefCell<CanvasConnection>>>,
	connection_ids: Vec<Uuid>,
	#[serde(skip)]
	canvas_rect: Rect,
}

impl Default for CanvasBoard {
	fn default() -> Self {
		Self {
			id: Uuid::default(),
			board: Board::default(),
			texture_handle: None,
			display_size: Vec2::ZERO,
			image_rect: Rect::ZERO,
			pin_locations: Vec::new(),
			canvas_pos: Vec2::ZERO,
			connection_ids: Vec::new(),
			connections: Vec::new(),
			canvas_rect: Rect::ZERO,
		}
	}
}

impl CanvasBoard {
	pub fn new(board: &Board) -> Option<Self> {
		if let Some(svg_board_info) = &board.svg_board_info {
			let display_size = svg_board_info.physical_size;
			let image_origin = egui::pos2(0.0, 0.0);
			let image_rect = Rect::from_min_size(image_origin, display_size);

			let mut pin_locations = Vec::new();

			for (pin_name, mut pin_rect) in &svg_board_info.pin_rects {
				// translate the rects so they are in absolute coordinates
				pin_rect = pin_rect.translate(image_rect.left_top().to_vec2());
				pin_locations.push((pin_name.clone(), pin_rect));
			}

			let canvas_rect = Rect::ZERO;

			Some(Self {
				id: Uuid::new_v4(),
				board: board.clone(),
				texture_handle: None,
				display_size,
				image_rect,
				pin_locations,
				canvas_pos: Vec2::new(0.0, 0.0),
				connections: Vec::new(),
				connection_ids: Vec::new(),
				canvas_rect,
			})
		} else {
			None
		}
	}

	pub fn init_refs(&mut self, kb: &Vec<Rc<RefCell<Board>>>, p: &Project) {
		if let Some(kb_board) = kb.iter().find(|b_rc| {
			let b = b_rc.borrow();
			b.get_name() == self.board.get_name()
		}) {
			self.board = kb_board.borrow().clone();
		}

		if let Some(svg_board_info) = &self.board.svg_board_info {
			println!("found board svg info on load");
			let display_size = svg_board_info.physical_size;
			let image_origin = egui::pos2(0.0, 0.0);
			self.image_rect = Rect::from_min_size(image_origin, display_size);
		}

		self.connections = self.connection_ids.iter()
			.filter_map(|c_id| {
				p.connections_iter()
					.find(|c| c.borrow().id == *c_id)
					.map(|c| c.clone())
			})
			.collect();

		println!("populated board {} with {} connections", self.board.get_name(), self.connections.len());
	}

	pub fn draw(&mut self, ui: &mut egui::Ui, to_screen: &RectTransform, mouse_pos: &Pos2) {
		self.canvas_update(to_screen);
		if self.texture_handle.is_none() {
			if let Some(svg_board_info) = &self.board.svg_board_info {
				self.texture_handle = Some(ui.ctx().load_texture(self.board.get_name(), svg_board_info.image.clone(), Default::default()));
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

	pub fn draw_pins(&mut self, ui: &mut egui::Ui, to_screen: &RectTransform, mouse_pos: &Pos2, draw_all_pins: bool) {
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
		ui.painter().rect(
			self.canvas_rect,
			10,
			Color32::from_rgba_unmultiplied(0, 0, 127, 63),
			egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 63)),
			egui::StrokeKind::Outside
		);
	}

	pub fn draw_pin(&self, ui: &mut egui::Ui, pin_name: &String, pin_rect: &Rect) {
		let pin_name_color = Color32::from_rgba_unmultiplied(0, 255, 0, 63);
		let pin_color = Color32::from_rgba_unmultiplied(0, 255, 0, 255);

		let pin_r = pin_rect.height() / 2.0;

		ui.painter().circle_filled(
			pin_rect.center(),
			pin_r,
			pin_color,
		);

		let text_rect = ui.painter().text(
			pin_rect.center()+Vec2{x:pin_r+2.0,y:0.0},
			egui::Align2::LEFT_CENTER,
			format!("{}", &pin_name),
			egui::FontId::monospace(pin_r*2.0),
			pin_name_color,
		);
	}

	pub fn canvas_update(&mut self, to_screen: &RectTransform) {
		let canvas_rect = self.image_rect.translate(self.canvas_pos);
		self.canvas_rect = to_screen.transform_rect(canvas_rect);
	}

	pub fn contains(&self, to_screen: &RectTransform, mouse_pos: &Pos2) -> bool {
		if (self.canvas_rect.contains(*mouse_pos)) {
			return true;
		}
		return false;
	}

	pub fn interact(&mut self, to_screen: &RectTransform, zoom: &f32, response: &Response, mouse_pos: &Pos2) -> bool {
		if self.contains(to_screen, mouse_pos) {
			if response.clicked() {
				return true;
			}
	
			if response.dragged() {
				if !self.connections.is_empty() {
					return false;
				}
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

	pub fn get_pin_location(&self, pin_name: &String) -> Option<Pos2> {
		self.pin_locations
			.iter()
			.find(|(name, _rect)| name == pin_name)
			.map(|(_name, rect)| rect.center())
	}

	pub fn get_canvas_position(&self) -> Vec2 {
		return self.canvas_pos;
	}

	pub fn drop_connection(&mut self, r: &Rc<RefCell<CanvasConnection>>) {
		self.connections.retain(|c| !Rc::ptr_eq(c, r));
		self.connection_ids.retain(|c|  *c != r.borrow().id);
	}

	pub fn add_connection(&mut self, r: &Rc<RefCell<CanvasConnection>>) {
		self.connection_ids.push(r.borrow().id);
		self.connections.push(r.clone());
	}
}


#[cfg(test)]
mod tests {
    use crate::app::{CanvasBoard, CanvasConnection};
	use crate::board::{Board, svg_reader::SvgBoardInfo};
	use crate::project::Project;
    use std::rc::Rc;
    use std::cell::RefCell;
	use uuid::Uuid;

	impl Board {
		pub fn dummy_svg() -> Self {
			let mut board = Board::default();
			board.svg_board_info = Some(SvgBoardInfo::default());
			board
		}
	}

    #[test]
    fn test_board_connection_uuids() {
		let mut project = Project::default();

		// board rcs
		let b1_rc = Rc::new(RefCell::new(Board::dummy_svg()));
		let b2_rc = Rc::new(RefCell::new(Board::dummy_svg()));
		let kb = vec![b1_rc.clone(), b2_rc.clone()];
		
		// canvas board rcs
		let cb1_rc = project.add_board(&b1_rc).unwrap();
		let cb2_rc = project.add_board(&b2_rc).unwrap();

		let cb1_id = cb1_rc.borrow().id;
		let cb2_id = cb2_rc.borrow().id;

		// connection
		let connection_rc = Rc::new(RefCell::new(CanvasConnection::new(cb1_rc, "test".to_string())));
		connection_rc.borrow_mut().end(cb2_rc, "test2".to_string());
		project.add_connection(&connection_rc);

		// grab the boards back from the connections after UUIDs generated
		let conn = project.connections.first().unwrap().borrow();
		let mut sb_id = conn.get_start_board().borrow().id;
		let mut eb_id = conn.get_end_board().unwrap().borrow().id;

		// ensure the UUIDs stored within the project reflect objects
		// if so, deserialization will pick them back up and generate runtime references
		assert_eq!(sb_id, cb1_id);
		assert_eq!(eb_id, cb2_id);
	}

	use serde_json;

	#[test]
	fn test_board_connection_references() {
		let mut project = Project::default();

		// board rcs
		let b1_rc = Rc::new(RefCell::new(Board::dummy_svg()));
		let b2_rc = Rc::new(RefCell::new(Board::dummy_svg()));
		let kb = vec![b1_rc.clone(), b2_rc.clone()];
		
		// canvas board rcs
		let cb1_rc = project.add_board(&b1_rc).unwrap();
		let cb2_rc = project.add_board(&b2_rc).unwrap();

		// connection
		let connection_rc = Rc::new(RefCell::new(CanvasConnection::new(cb1_rc.clone(), "test".to_string())));
		connection_rc.borrow_mut().end(cb2_rc.clone(), "test2".to_string());
		project.add_connection(&connection_rc);

		let serialized = serde_json::to_string(&project).expect("Could not serialize.");
		let mut deserialized: Project = serde_json::from_str(&serialized).expect("Could not deserialize.");
	
		for b in deserialized.boards_iter() {
			b.borrow_mut().init_refs(&kb, &deserialized);
		}

		// grab the boards back from the connections after UUIDs generated
		let conn = deserialized.connections.first().unwrap().borrow();
		let sb_rc = conn.get_start_board();
		let eb_rc = conn.get_end_board().unwrap();

		// ensure the UUIDs stored within the project reflect objects
		// if so, deserialization will pick them back up and generate runtime references
		assert!(Rc::ptr_eq(&sb_rc, &cb1_rc));
		assert!(Rc::ptr_eq(&eb_rc, &cb2_rc));
	}
}