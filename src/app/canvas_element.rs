use crate::app::{CanvasBoard, CanvasConnection};
use egui::{Color32, Pos2, Rect, Response, Ui, Vec2};
use emath::RectTransform;
use uuid::Uuid;
use std::collections::HashMap;

use std::cell::RefCell;
use std::rc::Rc;

pub enum CanvasSelection {
    Board(Rc<RefCell<CanvasBoard>>),
    Connection(Rc<RefCell<CanvasConnection>>),
    /// A group of connections with specific roles such that as a group, they form a protocol (e.g. I2C)
    ProtocolGroup {
        group_id: Uuid,
        connections: Vec<Rc<RefCell<CanvasConnection>>>,
    },
    /// Select wire from group: Individual connection selected within a protocol group
    /// Shows group border + highlights the specific connection
    WithinProtocolGroup {
        group_id: Uuid,
        all_connections: Vec<Rc<RefCell<CanvasConnection>>>,
        selected_connection: Rc<RefCell<CanvasConnection>>,
    },
}

impl CanvasSelection {
    pub fn interact(
        &mut self,
        to_screen: &RectTransform,
        zoom: &f32,
        response: &Response,
        mouse_pos: &Pos2,
    ) -> bool {
        match self {
            CanvasSelection::Board(b) => b
                .borrow_mut()
                .interact(to_screen, zoom, response, mouse_pos),
            CanvasSelection::Connection(c) => c
                .borrow_mut()
                .interact(to_screen, zoom, response, mouse_pos),
            CanvasSelection::ProtocolGroup { connections, .. } => {
                // Allow interaction with any connection in the group
                let mut interacted = false;
                for conn in connections {
                    if conn
                        .borrow_mut()
                        .interact(to_screen, zoom, response, mouse_pos)
                    {
                        interacted = true;
                    }
                }
                interacted
            }
            CanvasSelection::WithinProtocolGroup { selected_connection, .. } => {
                // Only the selected connection is interactive in select single wire mode
                selected_connection
                    .borrow_mut()
                    .interact(to_screen, zoom, response, mouse_pos)
            }
        }
    }

    pub fn contains(&self, to_screen: &RectTransform, mouse_pos: &Pos2) -> bool {
        match self {
            CanvasSelection::Board(b) => b.borrow().contains(to_screen, mouse_pos),
            CanvasSelection::Connection(c) => c.borrow().contains(to_screen, mouse_pos),
            CanvasSelection::ProtocolGroup { connections, .. } => {
                // Check if mouse is over any connection in the group
                connections
                    .iter()
                    .any(|conn| conn.borrow().contains(to_screen, mouse_pos))
            }
            CanvasSelection::WithinProtocolGroup { selected_connection, .. } => {
                // Only check the selected connection
                selected_connection.borrow().contains(to_screen, mouse_pos)
            }
        }
    }

    pub fn highlight(&self, ui: &mut egui::Ui, to_screen: &RectTransform, colorscheme: &HashMap<String, Color32>) {
        match self {
            CanvasSelection::Board(b) => b.borrow().highlight(ui, to_screen),
            CanvasSelection::Connection(c) => c.borrow().highlight(ui, to_screen),
            CanvasSelection::ProtocolGroup { .. } => {
                // Group highlighting is drawn earlier in the render loop (before connections)
                // Here we do not highlight individual connections to avoid double-drawing
            }
            CanvasSelection::WithinProtocolGroup { selected_connection, .. } => {
                // Group highlighting is drawn earlier in render loop
                // Here we just highlight the individual connection's waypoints
                selected_connection.borrow().highlight(ui, to_screen);
            }
        }
    }
}
