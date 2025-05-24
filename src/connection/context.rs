use std::collections::HashMap;

use crate::messages::ModbusQuery;
//This struct is meant to hold the state of the on going modbus communication
pub struct ModbusContext {
    pub queued_queries: Vec<ModbusQuery>,
    pub on_going_queries: HashMap<u16, ModbusQuery>,
}

impl ModbusContext {
    pub fn new() -> Self {
        ModbusContext {
            queued_queries: Vec::new(),
            on_going_queries: HashMap::new(),
        }
    }
}
