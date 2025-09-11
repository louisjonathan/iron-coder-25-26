use crate::project::Connection;
use egui::{Pos2, Vec2, Color32, Rect};
use emath::RectTransform;

pub struct CanvasConnection {
	// connection: Connection,
	points: Vec<Pos2>,
	color: egui::Color32,
	width: f32,
}

impl CanvasConnection {
	pub fn new() -> Self {
		let color = egui::Color32::RED;
		let points = Vec::<Pos2>::new();
		let width = 4.0;
		Self {
			points,
			color,
			width,
		}
	}

	pub fn draw(&self, ui: &mut egui::Ui, to_screen: &RectTransform) {
		if self.points.len() > 1 {
			for i in 0..self.points.len() - 1 {
				let start = self.points[i];
				let end = self.points[i + 1];
				ui.painter().line_segment(
					[start, end],
					egui::Stroke::new(self.width, self.color),
				);
			}
		}
	}

	pub fn add_point(&mut self, p: Pos2)
	{
		self.points.push(p);
	}
}