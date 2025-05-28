use crate::messages::{ModbusMessageData, FunctionCode};
use crate::common::{ModbusDataType, ModbusTable};
use crate::codec::ModbusSerialize;

mod rtu;
mod rtu_over_tcp;
mod tcp;

#[derive(Clone, PartialEq, Debug)]
pub struct ReadQueryParameters {
    pub table: ModbusTable,
    pub starting_address: u16,
    pub ammount: u16,
}
#[derive(Clone, PartialEq, Debug)]
pub struct SingleWriteQueryParameters {
    pub table: ModbusTable,
    pub starting_address: u16,
    pub value: ModbusDataType,
}
#[derive(Clone, PartialEq, Debug)]
pub struct MultipleWriteQueryParameters {
    pub table: ModbusTable,
    pub starting_address: u16,
    pub values: Vec<ModbusDataType>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct MultipleReadWriteQueryParameters {
    pub table: ModbusTable,
    pub read_starting_address: u16,
    pub read_ammount: u16,
    pub write_starting_address: u16,
    pub values: Vec<ModbusDataType>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ModbusQuery {
    ReadQuery {
        message_data: ModbusMessageData,
        params: ReadQueryParameters,
    },
    SingleWriteQuery {
        message_data: ModbusMessageData,
        params: SingleWriteQueryParameters,
    },
    MultipleWriteQuery {
        message_data: ModbusMessageData,
        params: MultipleWriteQueryParameters,
    },
    MultipleReadWriteQuery {
        message_data: ModbusMessageData,
        params: MultipleReadWriteQueryParameters,
    },
}

impl ModbusSerialize for ModbusQuery {}

impl ModbusQuery {
    pub fn get_message_data(&self) -> &ModbusMessageData {
        match self {
            ModbusQuery::ReadQuery {
                message_data,
                params: _params,
            } => message_data,
            ModbusQuery::SingleWriteQuery {
                message_data,
                params: _params,
            } => message_data,
            ModbusQuery::MultipleWriteQuery {
                message_data,
                params: _params,
            } => message_data,
            ModbusQuery::MultipleReadWriteQuery {
                message_data,
                params: _params,
            } => message_data,
        }
    }
}
