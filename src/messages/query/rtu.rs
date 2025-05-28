use crate::messages::ModbusQuery;
use crate::codec::rtu::ModbusRtuSerialize;

use anyhow::{Result, anyhow};

impl ModbusRtuSerialize for ModbusQuery {
    fn rtu_deserialize(_data: Vec<u8>) -> Result<Vec<Self>> {
        Err(anyhow!("Not implemented"))
    }

    fn rtu_serialize(&self) -> Result<Vec<u8>> {
        Err(anyhow!("Not implemented"))
    }
}