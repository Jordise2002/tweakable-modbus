pub mod query;
pub mod response;

use anyhow::{anyhow, Result};
use num_enum::TryFromPrimitive;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ModbusTable {
    DiscreteInput,
    Coils,
    InputRegisters,
    HoldingRegisters,
}

impl ModbusTable {
    pub fn get_table_from_function_code(function_code: FunctionCode) -> Option<ModbusTable> {
        match function_code {
            FunctionCode::WriteSingleCoil
            | FunctionCode::ReadCoils
            | FunctionCode::WriteMultipleCoils => Some(ModbusTable::Coils),
            FunctionCode::WriteSingleHoldingRegister
            | FunctionCode::ReadMultipleHoldingRegister
            | FunctionCode::WriteMultipleHoldingRegisters
            | FunctionCode::ReadWriteMultipleRegisters => Some(ModbusTable::HoldingRegisters),
            FunctionCode::ReadInputRegisters => Some(ModbusTable::InputRegisters),
            FunctionCode::ReadDiscreteInputs => Some(ModbusTable::DiscreteInput),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ModbusDataType {
    Coil(bool),
    Register(u16),
}

impl ModbusDataType {
    pub fn get_representation(&self) -> u16 {
        match self {
            ModbusDataType::Coil(value) => {
                if *value {
                    0xFF00
                } else {
                    0x0000
                }
            }
            ModbusDataType::Register(value) => *value,
        }
    }

    pub fn coil_from_representation(raw_value: u16) -> Result<Self>
    {
        match raw_value {
            0xFF00 => Ok(ModbusDataType::Coil(true)),
            0x0000 => Ok(ModbusDataType::Coil(false)),
            _ => Err(anyhow!("{} can be decoded to a coil, only valid values are 0xFF00 and 0x0000", raw_value))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum FunctionCode {
    ReadCoils = 1,
    ReadDiscreteInputs = 2,
    ReadMultipleHoldingRegister = 3,
    ReadInputRegisters = 4,
    WriteSingleCoil = 5,
    WriteSingleHoldingRegister = 6,
    ReadExceptionStatus = 7,  //RTU ONLY
    Diagnostic = 8,           //RTU ONLY
    GetCommEventCounter = 11, //RTU ONLY
    GetCommEventLog = 12,     //RTU ONLY
    WriteMultipleCoils = 15,
    WriteMultipleHoldingRegisters = 16,
    ReportServerID = 17, //RTU ONLY
    ReadFileRecord = 20,
    WriteFileRecord = 21,
    MaskWriteRegister = 22,
    ReadWriteMultipleRegisters = 23,
    ReadFIFOQueue = 24,
    ReadDeviceIdentification = 43,
    NoFunctionCode = 0xFF,
}

#[derive(Clone, Copy, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum ExceptionCode {
    IllegalFunction = 1,
    IllegalDataAddress = 2,
    IllegalDataValue = 3,
    ServerDeviceFailure = 4,
    Acknowledge = 5,
    ServerDeviceBusy = 6,
    MemoryParityError = 8,
    GatewayPathUnavailable = 0xA,
    GatewayTargetDeviceFailedToRespond = 0xB,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ModbusMessageData {
    pub slave_id: u8,
    pub function_code: FunctionCode,
    pub transaction_id: u16,
}

pub use query::ModbusQuery;
pub use response::ModbusResponse;
