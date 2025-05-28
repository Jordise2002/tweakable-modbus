use crate::common::{ModbusDataType, ModbusTable};
use crate::messages::{FunctionCode, ExceptionCode, ModbusMessageData};
use crate::codec::ModbusSerialize;

mod rtu;
mod rtu_over_tcp;
mod tcp;

#[derive(Clone, PartialEq, Debug)]
pub struct ReadResponseParameters {
    pub table: ModbusTable,
    pub values: Vec<ModbusDataType>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SingleWriteResponseParameters {
    pub table: ModbusTable,
    pub address: u16,
    pub value: ModbusDataType,
}

#[derive(Clone, PartialEq, Debug)]
pub struct MultipleWriteResponse {
    pub table: ModbusTable,
    pub address: u16,
    pub ammount: u16,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ModbusResponse {
    ReadResponse {
        message_data: ModbusMessageData,
        params: ReadResponseParameters,
    },
    SingleWriteResponse {
        message_data: ModbusMessageData,
        params: SingleWriteResponseParameters,
    },
    MultipleWriteResponse {
        message_data: ModbusMessageData,
        params: MultipleWriteResponse,
    },

    Error {
        message_data: ModbusMessageData,
        exception_code: ExceptionCode,
    },
}

impl ModbusSerialize for ModbusResponse {}

impl ModbusResponse {
    pub fn get_message_data(&self) -> &ModbusMessageData {
        match self {
            ModbusResponse::ReadResponse {
                message_data,
                params: _params,
            } => message_data,
            ModbusResponse::SingleWriteResponse {
                message_data,
                params: _params,
            } => message_data,
            ModbusResponse::MultipleWriteResponse {
                message_data,
                params: _params,
            } => message_data,
            ModbusResponse::Error {
                message_data,
                exception_code: _exception_code,
            } => message_data,
        }
    }
}
