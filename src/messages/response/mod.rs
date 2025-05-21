use super::*;
use crate::codec::ModbusSerialize;

mod tcp;
mod rtu;
mod rtu_over_tcp;

#[derive(Clone, PartialEq, Debug)]
pub struct ReadResponseParameters
{
    table: ModbusTable,
    values: Vec<ModbusDataType>
}

#[derive(Clone, PartialEq, Debug)]
pub struct SingleWriteResponseParameters
{
    table: ModbusTable,
    address: u16,
    value: ModbusDataType
}

#[derive(Clone, PartialEq, Debug)]
pub struct MultipleWriteResponse
{
    table: ModbusTable,
    address: u16,
    ammount: u16
}

#[derive(Clone,PartialEq,Debug)]
pub enum ModbusResponse 
{
    ReadResponse{message_data: ModbusMessageData, params: ReadResponseParameters},
    SingleWriteResponse{message_data: ModbusMessageData, params: SingleWriteResponseParameters},
    MultipleWriteResponse{message_data: ModbusMessageData, params: MultipleWriteResponse},

    Error{message_data : ModbusMessageData, exception_code: ExceptionCode}
}

impl ModbusSerialize for ModbusResponse {}