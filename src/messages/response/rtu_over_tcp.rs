use crate::codec::rtu_over_tcp::ModbusRtuOverTcpSerialize;

use super::*;

impl ModbusRtuOverTcpSerialize for ModbusResponse
{
    fn rtu_over_tcp_deserialize(_data: Vec<u8>) -> Result<Vec<Self>> {
        Err(anyhow!("Not implemented!"))   
    }

    fn rtu_over_tcp_serialize(&self) -> Result<Vec<u8>> {
        Err(anyhow!("Not implemented!"))
    }
}