use eframe::glow::NativeShader;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::pin;
use std::rc::Rc;
use uuid::Uuid;

use crate::app::CanvasConnection;
use crate::board::Board;
use crate::board::Pin;
use crate::project::Project;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WizardType {
    None,
    I2C,
    SPI,
    UART,
}

impl WizardType {
    pub fn display_name(&self) -> &'static str {
        match self {
            WizardType::None => "None.",
            WizardType::I2C => "I2C Wiz.",
            WizardType::SPI => "SPI Wiz.",
            WizardType::UART => "UART Wiz.",
        }
    }

    pub fn required_roles(&self) -> Vec<(String, String)> {
        match self {
            WizardType::None => vec![],
            // main board, peripheral board
            WizardType::I2C => vec![
                ("SDA".to_string(), "SDA".to_string()),
                ("SCL".to_string(), "SCL".to_string()),
            ],
            WizardType::SPI => vec![
                ("MOSI".to_string(), "MISO".to_string()),
                ("MISO".to_string(), "MOSI".to_string()),
                ("SCK".to_string(), "SCK".to_string()),
                ("CS".to_string(), "CS".to_string()),
            ],
            WizardType::UART => vec![
                ("TX".to_string(), "RX".to_string()),
                ("RX".to_string(), "TX".to_string()),
            ],
        }
    }
}

#[derive(Clone)]
pub enum WizardState {
    Uninitialized,
    SelectingRole {
        current_role_index: usize,
        pins_left_to_connect: u8,
        selected_pins: Vec<(String, u32, String, Uuid)>, // (role_name, pin_number, pin_name, board_id)
        /// Connections created during the wizard
        created_connections: Vec<Rc<RefCell<CanvasConnection>>>,
    },
    Complete {
        /// Connections created during the wizard
        created_connections: Vec<Rc<RefCell<CanvasConnection>>>,
    },
    Review {
        protocol_group_id: Uuid,
        /// Roles that still need connections (main_role, peripheral_role)
        missing_roles: Vec<(String, String)>,
        /// Existing connections in the group: role -> connection
        existing_connections: HashMap<String, Rc<RefCell<CanvasConnection>>>,
    },
}

pub struct ConnectionWizard {
    pub wizard_type: WizardType,
    pub state: WizardState,
    pub error_message: Option<String>,
    pub tooltip_message: Option<String>,
    required_roles: Vec<(String, String)>,
}
impl ConnectionWizard {
    /// Create a new wizard for the given type
    pub fn new(wizard_type: WizardType, board: &Board) -> Self {
        let mut wizard = Self {
            wizard_type,
            state: WizardState::Uninitialized,
            error_message: None,
            tooltip_message: None,
            required_roles: wizard_type.required_roles(),
        };
        if wizard_type != WizardType::None {
            wizard.advance_to_role(0, board);
        }
        wizard
    }

    /// Get the currently required role name
    pub fn current_role(&self) -> Option<String> {
        match &self.state {
            WizardState::SelectingRole {
                current_role_index,
                pins_left_to_connect,
                ..
            } => {
                if let Some((role_main, role_peripheral)) =
                    self.required_roles.get(*current_role_index).cloned()
                {
                    match pins_left_to_connect {
                        1 => Some(role_peripheral),
                        2 => Some(role_main),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    /// Get all selected pins so far
    pub fn selected_pins(&self) -> Vec<(String, u32, String, Uuid)> {
        match &self.state {
            WizardState::SelectingRole { selected_pins, .. } => selected_pins.clone(),
            _ => vec![],
        }
    }

    /// Get all connections created during the wizard
    pub fn created_connections(&self) -> Vec<Rc<RefCell<CanvasConnection>>> {
        match &self.state {
            WizardState::SelectingRole {
                created_connections,
                ..
            } => created_connections.clone(),
            WizardState::Complete {
                created_connections,
            } => created_connections.clone(),
            _ => vec![],
        }
    }

    /// Add a connection to the wizard's tracking
    pub fn add_created_connection(&mut self, connection: Rc<RefCell<CanvasConnection>>) {
        match &mut self.state {
            WizardState::SelectingRole {
                created_connections,
                ..
            }
            | WizardState::Complete {
                created_connections,
                ..
            } => {
                created_connections.push(connection);
                println!("CONNECTION COMPLETE");
            }
            _ => {
                // Do nothing in other states
            }
        }
    }

    /// Given a selected pin, check if the pin is capable of fulfilling the current role
    pub fn can_select_pin(&mut self, pin_number: u32, board: &Board) -> bool {
        if let Some(current_role) = self.current_role() {
            match &mut self.state {
                WizardState::SelectingRole { .. } => {
                    if let Some(roles) = board.pinout.get_pin_roles(&pin_number) {
                        if roles.contains(&current_role) {
                            let pin_name = board
                                .pinout
                                .get_pin_name(&pin_number)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("Pin {}", pin_number));
                            self.tooltip_message =
                                Some(format!("Pick Me! ({}->{})", pin_name, current_role));
                            return true;
                        } else {
                            let pin_name = board
                                .pinout
                                .get_pin_name(&pin_number)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("Pin {}", pin_number));
                            self.tooltip_message =
                                Some(format!("{} cannot be used for {}", pin_name, current_role));
                        }
                    }
                    false
                }
                _ => {
                    self.tooltip_message = None;
                    false
                }
            }
        } else {
            self.tooltip_message = None;
            false
        }
    }

    /// Given a selected pin, return true if the pin is valid and was accepted
    /// Returns true and the wizard advances, returns false and shows error if invalid
    /// board_id is the UUID of the CanvasBoard that this pin belongs to
    pub fn handle_pin_selected(
        &mut self,
        pin_number: u32,
        board: &Board,
        board_id: Uuid,
        project: &mut Project,
    ) -> bool {
        if let Some(current_role) = self.current_role() {
            match &mut self.state {
                WizardState::SelectingRole {
                    current_role_index,
                    pins_left_to_connect,
                    selected_pins,
                    created_connections,
                } => {
                    if let Some(roles) = board.pinout.get_pin_roles(&pin_number) {
                        let pin_name = board
                            .pinout
                            .get_pin_name(&pin_number)
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");

                        if roles.contains(&current_role) {
                            selected_pins.push((
                                current_role.clone(),
                                pin_number,
                                board
                                    .pinout
                                    .get_pin_name(&pin_number)
                                    .unwrap_or(&"".to_string())
                                    .to_string(),
                                board_id,
                            ));
                            *pins_left_to_connect -= 1;

                            if *pins_left_to_connect == 0 {
                                // Completed a pin pair; Connection should be created by caller
                                *current_role_index += 1;

                                // Check if completed all roles
                                if *current_role_index >= self.required_roles.len() {
                                    // Move created_connections into the Complete state
                                    let conns = created_connections.clone();
                                    self.state = WizardState::Complete {
                                        created_connections: conns,
                                    };
                                    project.add_connection_bus(&self.wizard_type);
                                } else {
                                    *pins_left_to_connect = 2;
                                }
                            }
                            self.error_message = None;
                            return true;
                        } else {
                            let pin_name = board
                                .pinout
                                .get_pin_name(&pin_number)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("Pin {}", pin_number));
                            self.error_message =
                                Some(format!("Can't use {} for {}!", pin_name, current_role));
                        }
                    }
                    false
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Check if we just completed a pin pair (ready to create a connection)
    /// Call this after handle_pin_selected returns true
    pub fn should_create_connection(&self) -> bool {
        match &self.state {
            WizardState::SelectingRole { selected_pins, .. } => {
                // We should create a connection when we have an even number of selected pins (pairs)
                // and the count is greater than the number of created connections
                let pairs_selected = selected_pins.len() / 2;
                let pairs_created = self.created_connections().len();
                pairs_selected > pairs_created
            }
            _ => false,
        }
    }
    /// Handle undo - go back ONE selection (used internally for partial undo)
    fn handle_undo_one_selection(&mut self) -> bool {
        match &mut self.state {
            WizardState::SelectingRole {
                current_role_index,
                pins_left_to_connect,
                selected_pins,
                created_connections,
            } => {
                if selected_pins.is_empty() {
                    self.state = WizardState::Uninitialized;
                    return false;
                }

                // Remove last selection
                selected_pins.pop();
                *pins_left_to_connect += 1;

                if *pins_left_to_connect > 2 {
                    // We've undone back past the start of this role pair
                    // This means we completed a connection and need to remove it
                    created_connections.pop();

                    *pins_left_to_connect = 2;
                    if *current_role_index > 0 {
                        *current_role_index -= 1;
                    }
                }
                self.error_message = None;
                true
            }
            _ => false,
        }
    }

    /// Handle undo - go back to previous role and remove last connection if one was completed
    pub fn handle_undo(&mut self, board: &Board) -> bool {
        self.handle_undo_one_selection()
    }

    /// Undo a full connection (both pins in the pair)
    pub fn undo_full_connection(&mut self) -> bool {
        match &self.state {
            WizardState::SelectingRole {
                selected_pins,
                created_connections,
                ..
            } => {
                if created_connections.is_empty() {
                    return self.handle_undo_one_selection();
                }
                let first_undo = self.handle_undo_one_selection();
                let second_undo = self.handle_undo_one_selection();

                first_undo && second_undo
            }
            _ => false,
        }
    }

    /// Cancel the wizard
    pub fn cancel(&mut self) {
        self.state = WizardState::Uninitialized;
        self.error_message = None;
    }

    /// Move to a specific role index
    fn advance_to_role(&mut self, role_index: usize, board: &Board) {
        let required_roles = self.wizard_type.required_roles();

        match &mut self.state {
            WizardState::Uninitialized => {
                self.state = WizardState::SelectingRole {
                    current_role_index: role_index,
                    pins_left_to_connect: 2,
                    selected_pins: vec![],
                    created_connections: vec![],
                };
            }
            WizardState::SelectingRole {
                current_role_index, ..
            } => {
                *current_role_index = role_index;
            }
            WizardState::Complete { .. } => {
                self.state = WizardState::SelectingRole {
                    current_role_index: role_index,
                    pins_left_to_connect: 2,
                    selected_pins: vec![],
                    created_connections: vec![],
                };
            }
            WizardState::Review { .. } => {
                // Transition from Review to SelectingRole
                self.state = WizardState::SelectingRole {
                    current_role_index: role_index,
                    pins_left_to_connect: 2,
                    selected_pins: vec![],
                    created_connections: vec![],
                };
            }
        }
        self.error_message = None;
    }

    /// Get a status message for display in the UI
    pub fn status_message(&self) -> String {
        if let Some(error) = &self.error_message {
            return error.clone();
        }

        match &self.state {
            WizardState::Uninitialized => "Select a wizard type".to_string(),
            WizardState::SelectingRole {
                current_role_index,
                pins_left_to_connect,
                ..
            } => {
                let current_role = self.current_role().unwrap_or_default();
                let required_roles = self.wizard_type.required_roles();
                let board = match *pins_left_to_connect {
                    2 => "master board's",
                    1 => "peripheral board's",
                    _ => {
                        println!("error, invalid pins left to connect");
                        "[ERROR]"
                    }
                };

                format!(
                    "Step {}/{}: Select {} pin for {}",
                    current_role_index + 1,
                    required_roles.len(),
                    board,
                    current_role
                )
            }
            WizardState::Complete { .. } => "Wizard complete!".to_string(),
            WizardState::Review { missing_roles, .. } => {
                if missing_roles.is_empty() {
                    "Review: All connections complete!".to_string()
                } else {
                    format!(
                        "Review: {} missing connection{}",
                        missing_roles.len(),
                        if missing_roles.len() == 1 { "" } else { "s" }
                    )
                }
            }
        }
    }

    /// Create a wizard in Review mode for an existing protocol group
    /// This allows editing/completing partial protocol groups
    pub fn enter_review_mode(
        wizard_type: WizardType,
        protocol_group_id: Uuid,
        connections: Vec<Rc<RefCell<CanvasConnection>>>,
    ) -> Self {
        let required_roles = wizard_type.required_roles();

        // Build map of existing connections by role
        let mut existing_connections = HashMap::new();
        for conn in &connections {
            if let Some(role) = &conn.borrow().role {
                existing_connections.insert(role.clone(), conn.clone());
            }
        }

        // Find missing roles
        let missing_roles: Vec<(String, String)> = required_roles
            .iter()
            .filter(|(main_role, _)| {
                let is_missing = !existing_connections.contains_key(main_role);
                is_missing
            })
            .cloned()
            .collect();

        Self {
            wizard_type,
            state: WizardState::Review {
                protocol_group_id,
                missing_roles,
                existing_connections,
            },
            error_message: None,
            tooltip_message: None,
            required_roles,
        }
    }

    /// Get the missing roles from Review state
    pub fn missing_roles(&self) -> Vec<(String, String)> {
        match &self.state {
            WizardState::Review { missing_roles, .. } => missing_roles.clone(),
            _ => vec![],
        }
    }

    /// Get the existing connections from Review state
    pub fn existing_connections(&self) -> HashMap<String, Rc<RefCell<CanvasConnection>>> {
        match &self.state {
            WizardState::Review {
                existing_connections,
                ..
            } => existing_connections.clone(),
            _ => HashMap::new(),
        }
    }

    /// Start completing missing connections from Review mode
    /// Transitions from Review to SelectingRole for the first missing role
    pub fn start_completing_missing(&mut self, board: &Board) {
        if let WizardState::Review {
            missing_roles,
            existing_connections,
            ..
        } = &self.state
        {
            if missing_roles.is_empty() {
                return;
            }

            // Find the index of the first missing role in required_roles
            let first_missing = &missing_roles[0];
            if let Some(role_index) = self.required_roles.iter().position(|r| r == first_missing) {
                // Convert existing connections to Vec for SelectingRole state
                let created_connections: Vec<Rc<RefCell<CanvasConnection>>> =
                    existing_connections.values().cloned().collect();

                // Transition to SelectingRole for this role
                self.advance_to_role(role_index, board);

                // Restore the already-created connections
                if let WizardState::SelectingRole {
                    created_connections: conns,
                    ..
                } = &mut self.state
                {
                    for conn in created_connections {
                        conns.push(conn);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardResult {
    Complete,
    ContinueToNextRole,
    InvalidPin,
    Error,
}
