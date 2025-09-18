use crate::app::{CanvasBoard, CanvasConnection};
use egui::{Pos2, Rect, Ui, Vec2, Response};
use emath::RectTransform;

use std::rc::Rc;
use std::cell::RefCell;

pub enum CanvasSelection {
	Board(Rc<RefCell<CanvasBoard>>),
	Connection(Rc<RefCell<CanvasConnection>>),
}

impl CanvasSelection {
	pub fn interact(&mut self, to_screen: &RectTransform, zoom: &f32, response: &Response, mouse_pos: &Pos2) -> bool {
		match self {
			CanvasSelection::Board(b) => b.borrow_mut().interact(to_screen, zoom, response, mouse_pos),
			CanvasSelection::Connection(c) => c.borrow_mut().interact(to_screen, zoom, response, mouse_pos),
		}
	}

	pub fn contains(&self, to_screen: &RectTransform, mouse_pos: &Pos2) -> bool {
		match self {
			CanvasSelection::Board(b) => b.borrow().contains(to_screen, mouse_pos),
			CanvasSelection::Connection(c) => c.borrow().contains(to_screen, mouse_pos),
		}
	}

	pub fn highlight(&self, ui: &mut egui::Ui, to_screen: &RectTransform){
		match self {
			CanvasSelection::Board(b) => b.borrow().highlight(ui, to_screen),
			CanvasSelection::Connection(c) => c.borrow().highlight(ui, to_screen),
		}
	}
}