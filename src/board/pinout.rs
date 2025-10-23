// /// This module defines interfaces that a development board has
// use enum_iterator::Sequence;

// use syn;

// use serde::{Serialize, Deserialize};
// use std::fmt;

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Sequence)]
// #[non_exhaustive]
// pub enum InterfaceDirection {
//     Unknown,
//     Input,
//     Output,
//     Bidirectional,
// }

// impl fmt::Display for InterfaceDirection {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// #[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Sequence)]
// #[non_exhaustive]
// /// The various types of electrical interfaces we use with dev boards
// pub enum InterfaceType {
//     NONE,
//     GPIO,
//     ADC,
//     PWM,
//     UART,
//     I2C,
//     SPI,
// }

// impl fmt::Display for InterfaceType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Sequence)]
// pub struct Interface {
//     pub iface_type: InterfaceType,
//     pub direction: InterfaceDirection,
// }

// impl fmt::Display for Interface {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// impl Default for Interface {
//     fn default() -> Self {
//         Self {
//             iface_type: InterfaceType::NONE,
//             direction: InterfaceDirection::Unknown,
//         }
//     }
// }

// /// And InterfaceMapping is a map of an Interface to a set of pins on the Board.
// /// TODO: I think a "pin" should be able to be referenced by multiple different criteria,
// /// such as the "silkscreen labal", the physical pin number (i.e. counting around the board),
// /// the logical pin number, or possibly some other criteria.
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// #[serde(default)]
// pub struct InterfaceMapping {
//     pub interface: Interface,
//     pub pins: Vec<String>,
//     #[serde(skip)]
//     pub bsp_field: Option<syn::Field>,
// }

// impl Default for InterfaceMapping {
//     fn default() -> Self {
//         Self {
//             interface: Interface::default(),
//             pins: Vec::new(),
//             bsp_field: None,
//         }
//     }
// }

// /// A Pinout is a description of the available interfaces on a Board
// // #[derive(Serialize, Deserialize, Clone, Debug)]
// // pub struct Pinout {
// //     pinout: Vec<InterfaceMapping>,
// // }

// pub type Pinout = Vec<InterfaceMapping>;

// // impl Default for Pinout {
// //     fn default() -> Self {
// //         Self {
// //             pinout: Vec::new(),
// //         }
// //     }
// // }

// // impl Iterator for Pinout {
// //     type Item = InterfaceMapping;
// //     fn next(&self) -> Option<Self::Item> {
// //         self.pinout.next();
// //     }
// // }

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Pinout {
    #[serde(default)]
    pub interfaces: Vec<Interface>,
    #[serde(default)]
    pub pin_block: Option<PinBlock>,
    
    // Runtime computed fields (not in TOML)
    #[serde(skip)]
    pub pins: HashMap<u32, Rc<RefCell<Pin>>>,
    #[serde(skip)]
    pub interface_ref_map: HashMap<String, Rc<RefCell<Interface>>>,
}

// Implement Default for Pinout
impl Default for Pinout {
    fn default() -> Self {
        Self {
            interfaces: Vec::new(),
            pin_block: None,
            pins: HashMap::new(),
            interface_ref_map: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(default)]
pub struct Interface {
    pub name: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub bus: bool,
}

// impl Default for Interface {
//     fn default() -> Self {
//         Self {
//             name: String::new(),
//             roles: Vec::new(),
//             bus: false,
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinAssignment {
    #[serde(default)]
    numbers: Vec<u32>,
    #[serde(default)]
    start: Option<u32>,
    #[serde(default)]
    end: Option<u32>,
    #[serde(default)]
    exclude: Vec<u32>,
    #[serde(default)]
    interfaces: Vec<String>,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    alias_fmts: Vec<String>,
    #[serde(default)]
    notes: Option<String>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinBlock {
    #[serde(default)]
    pub ranges: Vec<PinAssignment>,
    #[serde(default)]
    pub assignments: Vec<PinAssignment>,
}

#[derive(Debug, Clone)]
pub struct InterfaceRef {
    pub name: Option<String>,
    pub role: Option<String>,
    pub interface_ref: Rc<RefCell<Interface>>,
}

#[derive(Debug, Clone)]
pub struct Pin {
    pub number: u32,
    pub interfaces: Vec<InterfaceRef>,
    pub notes: Vec<String>,
    pub aliases: Vec<String>,
}

impl PinAssignment {
    pub fn get_pin_numbers(&self) -> Vec<u32> {
        if !self.numbers.is_empty() {
            self.numbers.clone()
        } else if let (Some(start), Some(end)) = (self.start, self.end) {
            (start..=end)
                .filter(|pin_num| !self.exclude.contains(pin_num))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Pin {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            interfaces: Vec::new(),
            notes: Vec::new(),
            aliases: Vec::new(),
        }
    }
}

impl Pinout {
    pub fn process_config(&mut self) {
        let mut map = HashMap::new();

        for i in self.interfaces.iter().cloned() {
            let rc = Rc::new(RefCell::new(i));

            let name = rc.borrow().name.clone();
            map.insert(name, rc.clone());

            for role in rc.borrow().roles.iter().cloned() {
                map.insert(role, rc.clone());
            }
        }

        self.interface_ref_map = map;

        let mut block = match self.pin_block.take() {
            Some(block) => block,
            None => return,
        };

        for range in &block.ranges {
            self.process_pin_assignment(range);
        }
        for assignment in &block.assignments {
            self.process_pin_assignment(assignment);
        }

        self.pin_block = Some(block);
    }

    fn process_pin_assignment(&mut self, assignment: &PinAssignment) {
        for pin_num in assignment.get_pin_numbers() {
            let mut pin = self.pins.entry(pin_num).or_insert_with(|| Rc::new(RefCell::new(Pin::new(pin_num)))).borrow_mut();

            for alias_fmt in &assignment.alias_fmts {
                let alias = alias_fmt.replace("{}", &pin_num.to_string());
                pin.aliases.push(alias);
            }

            for interface_name in &assignment.interfaces {
                if let Some(interface_ref) = self.interface_ref_map.get(interface_name) {
                    pin.interfaces.push(InterfaceRef { 
                        name: Some(interface_name.clone()), 
                        role: None, 
                        interface_ref: interface_ref.clone() 
                    });
                }
            }

            for role_name in &assignment.roles {
                if let Some(interface_ref) = self.interface_ref_map.get(role_name) {
                    pin.interfaces.push(InterfaceRef { 
                        name: None, 
                        role: Some(role_name.clone()), 
                        interface_ref: interface_ref.clone() 
                    });
                }
            }

            if let Some(note) = &assignment.notes {
                pin.notes.push(note.clone());
            }
        }
    }
}