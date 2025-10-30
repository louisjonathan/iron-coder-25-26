use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Pinout {
    #[serde(default)]
    interfaces: Vec<Interface>,
    #[serde(default)]
    pin_block: Option<PinBlock>,
    
    #[serde(skip)]
    pub pins: HashMap<u32, Rc<RefCell<Pin>>>,
    #[serde(skip)]
    pub interface_ref_map: HashMap<String, Rc<RefCell<Interface>>>,
}

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
    pub notes: Vec<String>,
    pub aliases: Vec<String>,
    pub interfaces: Vec<InterfaceRef>,
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
                    println!("pin{} interface{}", pin_num, interface_name);
                }
            }
            
            for role_name in &assignment.roles {
                if let Some(interface_ref) = self.interface_ref_map.get(role_name) {
                    pin.interfaces.push(InterfaceRef { 
                        name: None, 
                        role: Some(role_name.clone()), 
                        interface_ref: interface_ref.clone() 
                    });
                    println!("pin{} role{}", pin_num, role_name);
                }
            }

            if let Some(note) = &assignment.notes {
                pin.notes.push(note.clone());
            }
        }
    }
}