use crate::messages::{FunctionCode, ModbusMessageData};

use anyhow::{anyhow, Result};
use byteorder::{BigEndian, ReadBytesExt};

use std::cell::Cell;
use std::io::Cursor;
pub trait ModbusTcpSerialize
where
    Self: Sized,
{
    fn tcp_serialize(&self) -> Result<Vec<u8>>;
    fn tcp_deserialize(data: Vec<u8>) -> Result<Vec<Self>>;
}

pub fn serialize_mbap(message_data: &ModbusMessageData, length: u16) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    let transaction_id = message_data
        .transaction_id
        .get()
        .ok_or_else(|| anyhow!("Trying to serialize a message without transaction id"))?;
    //Transaction Identifier
    result.extend_from_slice(&transaction_id.to_be_bytes());

    //Protocol Identifier: 0u16 means Modbus
    result.extend_from_slice(&0u16.to_be_bytes());

    //Length
    result.extend_from_slice(&length.to_be_bytes());

    //Slave Id
    result.push(message_data.slave_id);

    Ok(result)
}

pub fn deserialize_mbap(data: &mut Cursor<Vec<u8>>) -> Result<(ModbusMessageData, u16)> {
    let position = data.position() as usize;
    let size_left = data.get_ref().len() - position;

    if size_left < 7 {
        return Err(anyhow!(format!(
            "Not enough bytes to form an mbap: position {}, bytes left {}",
            position, size_left
        )));
    }

    let transaction_id = data.read_u16::<BigEndian>()?;

    let _protocol_id = data.read_u16::<BigEndian>()?;

    let length = data.read_u16::<BigEndian>()?;

    let slave_id = data.read_u8()?;

    Ok((
        ModbusMessageData {
            transaction_id: Cell::new(Some(transaction_id)),
            slave_id,
            function_code: FunctionCode::NoFunctionCode,
        },
        length - 1,
    ))
}
