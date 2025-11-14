use egui::accesskit::Role;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::rc::Rc;

#[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "lowercase")]
pub enum GPIODirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Pinout {
    pub interfaces: Vec<Interface>,
    pub pins: Vec<Pin>,
    #[serde(skip)]
    alias_map: HashMap<String, String>,
    #[serde(skip)]
    silkscreen_map: HashMap<u32, String>,
    #[serde(skip)]
    role_to_pins: HashMap<String, HashSet<u32>>,
    #[serde(skip)]
    pin_to_roles: HashMap<u32, HashSet<String>>,
    #[serde(skip)]
    role_to_interface: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Pin {
    pub physical: u32,
    #[serde(default)]
    pub logical: Option<u32>,
    pub silkscreen: String,
    #[serde(default)]
    pub roles: Vec<RoleAssignment>,
    #[serde(skip)]
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleAssignment {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub id: Option<u32>,
    #[serde(default)]
    pub direction: Option<GPIODirection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Interface {
    pub name: String,
    #[serde(default)]
    pub bus: bool,
    #[serde(default)]
    pub roles: Vec<InterfaceRole>,
    #[serde(default)]
    pub alias_fmt: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterfaceRole {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub alias_fmt: Option<String>,
}

impl Pinout {
    fn populate_silkscreen_map(&mut self) {
        for pin in &self.pins {
            self.silkscreen_map
                .insert(pin.physical, pin.silkscreen.clone());
        }
    }

    fn populate_interface_to_pins_map(&mut self, main_board: bool) {
        for pin in &self.pins {
            for role in &pin.roles {
                self.role_to_pins
                    .entry(role.name.clone())
                    .or_default()
                    .insert(pin.physical);
                self.pin_to_roles
                    .entry(pin.physical)
                    .or_default()
                    .insert(role.name.clone());

                if main_board {
                    if let Some(interface) = self.role_to_interface.get(&role.name) {
                        self.role_to_pins
                            .entry(interface.to_string())
                            .or_default()
                            .insert(pin.physical);
                        self.pin_to_roles
                            .entry(pin.physical)
                            .or_default()
                            .insert(interface.to_string());
                    } else {
                        if let Some(interface) =
                            self.interfaces.iter().find(|i| i.name == role.name)
                        {
                            for interface_role in &interface.roles {
                                self.role_to_pins
                                    .entry(interface_role.name.to_string())
                                    .or_default()
                                    .insert(pin.physical);
                                self.pin_to_roles
                                    .entry(pin.physical)
                                    .or_default()
                                    .insert(interface_role.name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    fn populate_alias_map(&mut self) {
        for interface in &self.interfaces {
            if let Some(fmt) = &interface.alias_fmt {
                self.alias_map
                    .insert(interface.name.clone(), fmt.to_string());
            }
            for role in &interface.roles {
                if let Some(fmt) = &role.alias_fmt {
                    self.alias_map.insert(role.name.clone(), fmt.to_string());
                }
                self.role_to_interface
                    .insert(role.name.clone(), interface.name.clone());
                // self.interface_to_roles.entry(interface.name.clone()).or_default().insert(role.name.clone());
            }
        }
    }

    fn populate_pin_aliases(&mut self) {
        self.populate_alias_map();
        for pin in &mut self.pins {
            for role in &pin.roles {
                let name = if let Some(fmt) = self.alias_map.get(&role.name) {
                    let id = role.id.or(pin.logical).unwrap_or_else(|| {
                        panic!(
                            "Neither role.id nor pin.logical exists for pin {:?} at role {:?}",
                            pin, role
                        );
                    });
                    fmt.replace("{}", &id.to_string())
                } else {
                    role.name.clone()
                };
                pin.aliases.insert(role.name.clone(), name.clone());
                if let Some(interface) = self.role_to_interface.get(&role.name) {
                    pin.aliases.insert(interface.clone(), name.clone());
                }
            }
        }
    }

    pub fn populate_pins(&mut self, main_board: bool) {
        self.populate_silkscreen_map();
        self.populate_pin_aliases();
        self.populate_interface_to_pins_map(main_board);
    }

    pub fn get_pin_name(&self, physical: &u32) -> Option<&String> {
        self.silkscreen_map.get(&physical)
    }

    pub fn get_pins_from_role(&self, role: &String) -> Option<&HashSet<u32>> {
        self.role_to_pins.get(role)
    }

    pub fn get_gpio_direction(&self, physical: &u32) -> Option<&GPIODirection> {
        let pin = self.pins.iter().find(|p| p.physical == *physical)?;
        let gpio_role = pin.roles.iter().find(|r| r.name == "GPIO")?;
        gpio_role.direction.as_ref()
    }

    pub fn get_peripheral_pin_interface(&self, physical: &u32) -> Option<&RoleAssignment> {
        let pin = self.pins.iter().find(|p| p.physical == *physical)?;
        pin.roles.first()
    }

    pub fn get_pin_alias(&self, physical: &u32, role: &String) -> Option<String> {
        let pin = self.pins.iter().find(|p| p.physical == *physical)?;
        pin.aliases.get(role).cloned()
    }

    pub fn get_pin_roles(&self, physical: &u32) -> Option<&HashSet<String>> {
        self.pin_to_roles.get(physical)
    }

    pub fn get_interface_from_role(&self, role: &String) -> Option<&String> {
        self.role_to_interface.get(role)
    }
}
