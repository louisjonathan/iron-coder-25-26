use crate::app::canvas_connection::CanvasConnection;
use crate::app::connection_wizard::WizardType;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

/// Represents a group of connections that form a complete protocol (I2C, SPI, UART, etc.)
/// This allows us to create, undo, and redo protocol connections as a single unit
#[derive(Clone, Serialize, Deserialize)]
pub struct CanvasProtocol {
    pub id: Uuid,
    pub protocol_type: WizardType,
    /// All the individual wire connections that make up this protocol
    /// For I2C: 2 connections (SDA, SCL)
    /// For SPI: 4 connections (MOSI, MISO, SCK, CS)
    /// For UART: 2 connections (TX, RX)
    #[serde(with = "crate::project::project::rc_refcell_vec")]
    pub connections: Vec<Rc<RefCell<CanvasConnection>>>,
}

impl CanvasProtocol {
    /// Create a new protocol connection group
    pub fn new(protocol_type: WizardType) -> Self {
        Self {
            id: Uuid::new_v4(),
            protocol_type,
            connections: Vec::new(),
        }
    }

    /// Add a connection to this protocol group
    pub fn add_connection(&mut self, connection: Rc<RefCell<CanvasConnection>>) {
        self.connections.push(connection);
    }

    /// Check if this protocol has all required connections
    pub fn is_complete(&self) -> bool {
        let expected_count = match self.protocol_type {
            WizardType::None => 0,
            WizardType::I2C => 2,   // SDA + SCL
            WizardType::SPI => 4,   // MOSI + MISO + SCK + CS
            WizardType::UART => 2,  // TX + RX
        };
        self.connections.len() == expected_count
    }

    /// Get all connections in this protocol
    pub fn get_connections(&self) -> &Vec<Rc<RefCell<CanvasConnection>>> {
        &self.connections
    }

    /// Remove all connections from this protocol
    pub fn clear(&mut self) {
        self.connections.clear();
    }

    /// Get the ID of this protocol group
    pub fn get_id(&self) -> Uuid {
        self.id
    }

    /// Assign this protocol group's ID to all connections in the group
    /// This creates the bidirectional link between connections and their parent group
    pub fn assign_to_connections(&self) {
        for conn in &self.connections {
            conn.borrow_mut().protocol_group_id = Some(self.id);
        }
    }

    /// Initialize references after deserialization
    /// Replaces deserialized connection Rc/RefCell objects with references to actual connections from the project
    pub fn init_refs(&mut self, project_connections: &Vec<Rc<RefCell<CanvasConnection>>>) {
        // Get the IDs of all connections in this group
        let connection_ids: Vec<Uuid> = self.connections
            .iter()
            .map(|c| c.borrow().id)
            .collect();

        // Clear the current connections (they're deserialized objects, not the real ones)
        self.connections.clear();

        // Find the actual connection objects from the project and add them
        for conn_id in connection_ids {
            if let Some(actual_conn) = project_connections.iter().find(|c| c.borrow().id == conn_id) {
                self.connections.push(actual_conn.clone());
            }
        }

        // Re-assign this group ID to all connections
        self.assign_to_connections();
    }
}
