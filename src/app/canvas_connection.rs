use crate::project::Connection;
use crate::board::Board;
use egui::{Pos2, Vec2, Color32, Rect};
use emath::RectTransform;
use crate::app::canvas_board::CanvasBoard;

use std::rc::Rc;
use std::cell::RefCell;

pub struct CanvasConnection {
    connection: Option<Connection>,
    points: Vec<Pos2>,
    color: egui::Color32,
    width: f32,
    start_board: Rc<RefCell<CanvasBoard>>,
    start_pin: String,
    end_board: Option<Rc<RefCell<CanvasBoard>>>,
    end_pin: Option<String>,
}

impl CanvasConnection {
    pub fn new(start_board: Rc<RefCell<CanvasBoard>>, start_pin: String) -> Self {
        let color = egui::Color32::RED;
        let points = Vec::<Pos2>::new();
        let width = 4.0;

		let mut points = points;
		// {
		// 	let canvas_board = start_board.borrow_mut();
		// 	if let Some(p) = canvas_board.get_pin_location(&start_pin) {
		// 		points.push(p);
		// 	}
		// }

        Self {
            connection: None,
            points,
            color,
            width,
            start_board: start_board,
            start_pin,
            end_board: None,
            end_pin: None,
        }
	}

	pub fn draw(&self, ui: &mut egui::Ui, to_screen: &RectTransform, mouse_pos: Pos2) {
		if self.points.len() > 1 {
			for i in 0..self.points.len() - 1 {
				let start = to_screen.transform_pos(self.points[i]);
				let end = to_screen.transform_pos(self.points[i+1]);
				ui.painter().line_segment(
					[start, end],
					egui::Stroke::new(self.width, self.color),
				);
			}
		}

		if self.end_pin == None {
			self.draw_ghost(ui, to_screen, mouse_pos);
		}
	}

	pub fn add_point(&mut self, p: Pos2)
	{
		let mut p = p;
		if let Some(lastp) = self.points.last() {
			let xdiff = (p.x - lastp.x).abs();
			let ydiff = (p.y - lastp.y).abs();
			if xdiff > ydiff {
				p.y = lastp.y;
			} else {
				p.x = lastp.x;
			}
		}
		self.points.push(p);
	}

	pub fn end(&mut self, end_board: Rc<RefCell<CanvasBoard>>, end_pin: String) {
		self.end_board = Some(end_board.clone());
		self.end_pin = Some(end_pin.clone());

		let sb = self.start_board.borrow();
		let eb = end_board.borrow();
		println!("CONNECTED {}:{} TO {}:{}", sb.board.get_name(), self.start_pin, eb.board.get_name(), end_pin);
	}

	pub fn draw_ghost(&self, ui: &mut egui::Ui, to_screen: &RectTransform, mouse_pos: Pos2)
	{
		if let Some(lastp) = self.points.last() {
			let mut p = mouse_pos;
			let xdiff = (p.x - lastp.x).abs();
			let ydiff = (p.y - lastp.y).abs();
			if xdiff > ydiff {
				p.y = lastp.y;
			} else {
				p.x = lastp.x;
			}

			let ghost_color = Color32::from_rgba_unmultiplied(
				self.color.r(),
				self.color.g(),
				self.color.b(),
				31
			);
			ui.painter().line_segment(
			[to_screen.transform_pos(*lastp), to_screen.transform_pos(p)],
			egui::Stroke::new(self.width, ghost_color));
		}
	}
}