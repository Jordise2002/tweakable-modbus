use super::*;
use crate::codec::rtu::ModbusRtuSerialize;

impl ModbusRtuSerialize for ModbusQuery {
    fn rtu_deserialize(_data: Vec<u8>) -> Result<Vec<Self>> {
        Err(anyhow!("Not implemented"))
    }

    fn rtu_serialize(&self) -> Result<Vec<u8>> {
        Err(anyhow!("Not implemented"))
    }
}