use crate::app::canvas_board::CanvasBoard;
use crate::app::canvas_connection::CanvasConnection;
use crate::app::canvas_protocol::CanvasProtocol;
use crate::project::Project;
use std::cell::RefCell;
use std::rc::Rc;

/// The Command trait defines the interface for all undoable/redoable operations
/// This is the Command pattern - each command knows how to execute itself and undo itself
pub trait Command {
    /// Execute this command, applying its changes to the project
    fn execute(&mut self, project: &mut Project);

    /// Undo this command, reverting its changes to the project
    fn undo(&mut self, project: &mut Project);

    /// Get a description of this command for debugging/UI
    fn description(&self) -> String;
}

/// Command to add a complete protocol connection (e.g., I2C with both SDA and SCL wires)
pub struct AddProtocolConnectionCommand {
    protocol_connection: CanvasProtocol,
}

impl AddProtocolConnectionCommand {
    pub fn new(protocol_connection: CanvasProtocol) -> Self {
        Self { protocol_connection }
    }
}

impl Command for AddProtocolConnectionCommand {
    fn execute(&mut self, project: &mut Project) {
        // Add all connections from the protocol to the project
        for conn in &self.protocol_connection.connections {
            project.add_connection(conn);
        }
    }

    fn undo(&mut self, project: &mut Project) {
        // Remove all connections that were part of this protocol
        println!("Protocol undo: Removing {} connections from {:?}",
                 self.protocol_connection.connections.len(),
                 self.protocol_connection.protocol_type);
        for conn in &self.protocol_connection.connections {
            project.remove_connection(conn);
        }
    }

    fn description(&self) -> String {
        format!("Add {:?} connection", self.protocol_connection.protocol_type)
    }
}

/// Command to remove a protocol connection
pub struct RemoveProtocolConnectionCommand {
    protocol_connection: CanvasProtocol,
}

impl RemoveProtocolConnectionCommand {
    pub fn new(protocol_connection: CanvasProtocol) -> Self {
        Self { protocol_connection }
    }
}

impl Command for RemoveProtocolConnectionCommand {
    fn execute(&mut self, project: &mut Project) {
        // Remove all connections
        for conn in &self.protocol_connection.connections {
            project.remove_connection(conn);
        }
    }

    fn undo(&mut self, project: &mut Project) {
        // Re-add all connections
        for conn in &self.protocol_connection.connections {
            project.add_connection(conn);
        }
    }

    fn description(&self) -> String {
        format!("Remove {:?} connection", self.protocol_connection.protocol_type)
    }
}

/// Manages the history of commands for undo/redo functionality
/// This uses two stacks: one for undo and one for redo
pub struct CommandHistory {
    /// Stack of commands that can be undone (most recent on top)
    undo_stack: Vec<Box<dyn Command>>,

    /// Stack of commands that can be redone (most recent undo on top)
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Execute a command and add it to the undo stack
    /// This clears the redo stack because we're starting a new "timeline"
    pub fn execute(&mut self, mut command: Box<dyn Command>, project: &mut Project) {
        command.execute(project);
        self.undo_stack.push(command);

        // Clear redo stack - we've started a new timeline
        self.redo_stack.clear();
    }

    /// Undo the most recent command
    /// Returns true if something was undone, false if undo stack is empty
    pub fn undo(&mut self, project: &mut Project) -> bool {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo(project);
            self.redo_stack.push(command);
            true
        } else {
            false
        }
    }

    /// Redo the most recently undone command
    /// Returns true if something was redone, false if redo stack is empty
    pub fn redo(&mut self, project: &mut Project) -> bool {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(project);
            self.undo_stack.push(command);
            true
        } else {
            false
        }
    }

    /// Check if we can undo (undo stack not empty)
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if we can redo (redo stack not empty)
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get the number of commands in the undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of commands in the redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Add a command to history without executing it (for commands that were already executed)
    /// This is useful when you want to track an action for undo/redo that has already been performed
    pub fn add_to_history(&mut self, command: Box<dyn Command>) {
        self.undo_stack.push(command);
        // Clear redo stack - we've started a new timeline
        self.redo_stack.clear();
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}
