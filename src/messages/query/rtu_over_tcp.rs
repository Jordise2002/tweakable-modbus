use super::*;
use crate::codec::rtu_over_tcp::ModbusRtuOverTcpSerialize;

impl ModbusRtuOverTcpSerialize for ModbusQuery {
    fn rtu_over_tcp_deserialize(data: Vec<u8>) -> Result<Vec<Self>> {
        Err(anyhow!("Not implemented"))
    }

    fn rtu_over_tcp_serialize(&self) -> Result<Vec<u8>> {
        Err(anyhow!("Not implemented"))
    }
}