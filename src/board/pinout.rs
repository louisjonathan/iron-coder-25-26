use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Pinout {
    pub interfaces: Vec<Interface>,
    pub pins: Vec<Pin>,
    #[serde(skip)]
    alias_map: HashMap<String, String>,
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
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleAssignment {
    #[serde(default)]
    name: String,
    #[serde(default)]
    id: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Interface {
    #[serde(default)]
    name: String,
    #[serde(default)]
    bus: bool,
    #[serde(default)]
    roles: Vec<InterfaceRole>,
    #[serde(default)]
    alias_fmt: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterfaceRole {
    #[serde(default)]
    name: String,
    #[serde(default)]
    alias_fmt: Option<String>,
}

impl Pinout {
    fn populate_alias_map(&mut self) {
        for interface in &self.interfaces {
            if let Some(fmt) = &interface.alias_fmt {
                self.alias_map.insert(interface.name.clone(), fmt.to_string());
            }
            for role in &interface.roles {
                if let Some(fmt) = &role.alias_fmt {
                    self.alias_map.insert(role.name.clone(), fmt.to_string());
                }
            }
        }
    }

    pub fn populate_pin_aliases(&mut self) {
        self.populate_alias_map();

        for pin in &mut self.pins {
            for role in &pin.roles {
                if let Some(fmt) = self.alias_map.get(&role.name) {
                    let id = role.id.or(pin.logical).unwrap_or_else(|| {
                        panic!("Neither role.id nor pin.logical exists for pin {:?} at role {:?}", pin, role);
                    });
                    pin.aliases.push(fmt.replace("{}", &id.to_string()));
                } else {
                    pin.aliases.push(role.name.clone());
                }

            }
        }
    }
}

