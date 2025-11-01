use crate::board::Board;
use egui::{Color32, Pos2, Rect, Response, Stroke, Vec2, Key, Ui};
use emath::RectTransform;
use crate::app::canvas_board::CanvasBoard;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::project::Project;

use std::rc::Rc;
use std::cell::RefCell;

use crate::board::Pin;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct CanvasConnection {
	pub name: String,
	pub id: Uuid,
	points: Vec<Pos2>,
	color: egui::Color32,

	#[serde(skip)]
	start_board: Rc<RefCell<CanvasBoard>>,
	start_board_id: Uuid,
	start_pin: u32,

	#[serde(skip)]
	end_board: Option<Rc<RefCell<CanvasBoard>>>,
	end_board_id: Uuid,
	end_pin: Option<u32>,

	#[serde(skip)]
	temp_name: String,
	#[serde(skip)]
	pub show_popup: bool
}

impl CanvasConnection {
	pub fn new(start_board: Rc<RefCell<CanvasBoard>>, start_pin: u32) -> Self {
		let color = egui::Color32::RED;
		let points = Vec::<Pos2>::new();

		let mut points = points;

		let start_board_id = start_board.borrow().id;

		Self {
			name: String::new(),
			id: Uuid::new_v4(),
			points,
			color,
			start_board: start_board,
			start_board_id,
			start_pin,
			end_board: None,
			end_board_id: Uuid::nil(),
			end_pin: None,
			temp_name: String::new(),
			show_popup: false,
		}
	}

	pub fn init_refs(&mut self, kb: &Vec<Rc<Board>>, p: &Project) {
		if let Some(s_b) = p.board_map.get(&self.start_board_id) {
			let id_copy = s_b.borrow().id;
			self.start_board = s_b.clone();
			self.start_board_id = id_copy;
		}
		if let Some(e_b) = p.board_map.get(&self.end_board_id) {
			let id_copy = e_b.borrow().id;
			self.end_board = Some(e_b.clone());
			self.end_board_id = id_copy;
		}
	}

	pub fn draw(&mut self, ui: &mut egui::Ui, to_screen: &RectTransform, mouse_pos: Pos2) {
		let len = self.points.len();
		if len > 0 && self.show_popup {
			let loc = self.points.last().unwrap();
			let loc = to_screen.transform_pos(*loc);
			self.creation_popup(ui, &loc);
		}
		if len > 1 {
			for i in 0..self.points.len() - 1 {
				let start = to_screen.transform_pos(self.points[i]);
				let end = to_screen.transform_pos(self.points[i+1]);
				ui.painter().line_segment(
					[start, end],
					egui::Stroke::new(4.0, self.color),
				);
			}
		}

		if self.end_pin == None {
			self.draw_ghost(ui, to_screen, mouse_pos);
		}
	}

	pub fn add_point(&mut self, mut p: Pos2) {
		// to avoid mut borrow, use index
		let len = self.points.len();

		if len == 0 {
			self.points.push(p);
			return;
		}

		let lastp = self.points[len - 1];

		let dx = p.x - lastp.x;
		let dy = p.y - lastp.y;

		if dx.abs() > dy.abs() {
			p.y = lastp.y;
			if len > 2 && self.points[len - 2].y == p.y {
				self.points[len - 1].x = p.x;
				return;
			}
		} else {
			p.x = lastp.x;
			if len > 2 && self.points[len - 2].x == p.x {
				self.points[len - 1].y = p.y;
				return;
			}
		}
		self.points.push(p);
	}

	pub fn add_end_point(&mut self, mouse_pos: &Pos2, pin_pos: Pos2) {
		
		self.add_point(pin_pos);
		self.add_point(pin_pos);
		
		// TODO: fix pin propagation
		// let tolerance = 10.0;
		// let len = self.points.len();
		// if self.points[len-1].distance(pin_pos) > tolerance || len == 2 {
		// 	self.add_point(pin_pos);
		// } else if len > 3 {
		// 	self.points[len-1] = pin_pos;
		// 	let dx = self.points[len-2].x - pin_pos.x;
		// 	let dy = self.points[len-2].y - pin_pos.y;
		// 	if dx.abs() < dy.abs() {
		// 		self.points[len-2].x = pin_pos.x;
		// 	} else {
		// 		self.points[len-2].y = pin_pos.y;
		// 	}
		// }
	}

	pub fn end(&mut self, board: Rc<RefCell<CanvasBoard>>, pin: u32) {
		let b = board.borrow();
		
		if b.board.as_ref().unwrap().is_main_board() {
			// we need to make start_board the main board to simplify things
			self.end_board = Some(self.start_board.clone());
			self.end_pin = Some(self.start_pin.clone());
			self.end_board_id = self.start_board_id;
			
			self.start_board = board.clone();
			self.start_pin = pin;
			self.start_board_id = b.id;
		} else {
			self.end_board = Some(board.clone());
			self.end_pin = Some(pin);
			self.end_board_id = b.id;
		}
		self.name = format!("{}_to_{}", self.start_pin, self.end_pin.clone().unwrap());
	}

	pub fn draw_ghost(&self, ui: &mut egui::Ui, to_screen: &RectTransform, mouse_pos: Pos2)
	{
		let mut p = mouse_pos;
		let len = self.points.len();
		if len > 0 {
			let lastp = self.points[len - 1];

			let dx = p.x - lastp.x;
			let dy = p.y - lastp.y;

			if dx.abs() > dy.abs() {
				p.y = lastp.y;
			} else {
				p.x = lastp.x;
			}
		}

		let ghost_color = Color32::from_rgba_unmultiplied(
			self.color.r(),
			self.color.g(),
			self.color.b(),
			31
		);
		ui.painter().line_segment(
		[to_screen.transform_pos(self.points[len - 1]), to_screen.transform_pos(p)],
		egui::Stroke::new(4.0, ghost_color));
	}

	pub fn highlight(&self, ui: &mut egui::Ui, to_screen: &RectTransform) {
		let pin_r = 5.0;
		let point_color = Color32::WHITE;
		// let point_color = Color32::from_rgba_unmultiplied(127, 0, 0, 191);
		for p in &self.points {
			let p_t = to_screen.transform_pos(*p);
			ui.painter().circle_stroke(
				p_t,
				pin_r,
				Stroke::new(2.0, point_color)
			);
		}
	}

	pub fn contains(&self, to_screen: &RectTransform, mouse_pos: &Pos2) -> bool {
		// make vec of points-1 rects
		// transform & check contains on each
		let padding = 10.0;

		for window in self.points.windows(2) {
			let p1 = to_screen.transform_pos(window[0]);
			let p2 = to_screen.transform_pos(window[1]);

			// vertical segments
			if (p1.x == p2.x) {
				let (ymin, ymax) = (p1.y.min(p2.y), p1.y.max(p2.y));
				if (mouse_pos.x - p1.x).abs() <= padding
					&& mouse_pos.y >= ymin - padding
					&& mouse_pos.y <= ymax + padding
				{
					return true;
				}
			}
			if (p1.y == p2.y) {
				let (xmin, xmax) = (p1.x.min(p2.x), p1.x.max(p2.x));
				if (mouse_pos.y - p1.y).abs() <= padding
					&& mouse_pos.x >= xmin - padding
					&& mouse_pos.x <= xmax + padding
				{
					return true;
				}
			}
		}
		return false;
	}

	pub fn interact(&mut self, to_screen: &RectTransform, zoom: &f32, response: &Response, mouse_pos: &Pos2) -> bool {
		// TODO: let you drag first or last points to other pins on the same board		
		let tolerance = 15.0;

		if response.dragged() {
			// rust borrow checker will not allow multiple neighboring mutations, index loop instead
			let len = self.points.len();
			for i in 1..len-1 {
				let dist = mouse_pos.distance(to_screen.transform_pos(self.points[i]));
				if dist < tolerance {
					let mut movement = response.drag_delta() / *zoom;

					if i == 1 {
						if self.points[i-1].x == self.points[i].x {
							movement.x = 0.0;
						} else {
							movement.y = 0.0;
						}
					}
					if i == len-2 {
						if self.points[i+1].x == self.points[i].x {
							movement.x = 0.0;
						} else {
							movement.y = 0.0;
						}
					}

					if i != 1 {
						if self.points[i-1].x == self.points[i].x {
							self.points[i-1].x += movement.x;
						} else {
							self.points[i-1].y += movement.y;
						}
					}

					if i != len-2 {
						if self.points[i+1].x == self.points[i].x {
							self.points[i+1].x += movement.x;
						} else {
							self.points[i+1].y += movement.y;
						}
					}

					self.points[i] += movement;
					return true;
				}
			}
		}

		return false;
	}

	pub fn get_start_board(&self) -> Rc<RefCell<CanvasBoard>> {
		return self.start_board.clone();
	}

	pub fn get_start_pin(&self) -> u32 {
		return self.start_pin.clone();
	}

	pub fn get_end_pin(&self) -> Option<u32> {
		return self.end_pin.clone();
	}

	pub fn get_end_board(&self) -> Option<Rc<RefCell<CanvasBoard>>> {
		return self.end_board.clone();
	}

	pub fn creation_popup(&mut self, ui: &Ui, p: &Pos2) {
		egui::show_tooltip_at(ui.ctx(), ui.layer_id(), egui::Id::new("my_tooltip"), *p, |ui| {
			ui.label("Configure Connection");
			ui.text_edit_singleline(&mut self.temp_name);
			if ui.button("Confirm").clicked() {
				self.show_popup = false;
				self.name = self.temp_name.clone();
			}
			if ui.button("Cancel").clicked() {
				self.show_popup = false;
			}
		});
	}
}
