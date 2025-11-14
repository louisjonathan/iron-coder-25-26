use eframe::glow::NativeShader;
use std::cell::RefCell;
use std::collections::HashSet;
use std::pin;
use std::rc::Rc;

use crate::board::Board;
use crate::board::pinout::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        selected_pins: Vec<(String, u32, String)>, // (role_name, pin_number)
    },
    Complete,
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
                selected_pins,
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
    pub fn selected_pins(&self) -> Vec<(String, u32, String)> {
        match &self.state {
            WizardState::SelectingRole { selected_pins, .. } => selected_pins.clone(),
            _ => vec![],
        }
    }
    /// Given a selected pin, check if the pin is capable of fulfilling the current role
    pub fn can_select_pin(&mut self, pin_number: u32, board: &Board) -> bool {
        if let Some(current_role) = self.current_role() {
            match &mut self.state {
                WizardState::SelectingRole {
                    current_role_index,
                    pins_left_to_connect,
                    selected_pins,
                } => {
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

    /// Given a selected pin, return true and add the pin to selected pins the pin if the pin is capable of the current required role and direction, otw. return false and do nothing,
    pub fn handle_pin_selected(&mut self, pin_number: u32, board: &Board) -> bool {
        if let Some(current_role) = self.current_role() {
            match &mut self.state {
                WizardState::SelectingRole {
                    current_role_index,
                    pins_left_to_connect,
                    selected_pins,
                } => {
                    if let Some(roles) = board.pinout.get_pin_roles(&pin_number) {
                        if roles.contains(&current_role) {
                            selected_pins.push((
                                current_role.clone(),
                                pin_number,
                                board
                                    .pinout
                                    .get_pin_name(&pin_number)
                                    .unwrap_or( &"".to_string())
                                    .to_string(),
                            ));
                            *pins_left_to_connect -= 1;

                            if *pins_left_to_connect == 0 {
                                *current_role_index += 1;

                                // Check if we've completed all roles
                                if *current_role_index >= self.required_roles.len() {
                                    self.state = WizardState::Complete;
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
    /// Handle undo - go back to previous role
    pub fn handle_undo(&mut self, board: &Board) -> bool {
        match &mut self.state {
            WizardState::SelectingRole {
                current_role_index,
                pins_left_to_connect,
                selected_pins,
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
                };
            }
            WizardState::SelectingRole {
                current_role_index, ..
            } => {
                *current_role_index = role_index;
            }
            WizardState::Complete => {
                self.state = WizardState::SelectingRole {
                    current_role_index: role_index,
                    pins_left_to_connect: 2,
                    selected_pins: vec![],
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
                selected_pins,
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
            WizardState::Complete => "Wizard complete!".to_string(),
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
