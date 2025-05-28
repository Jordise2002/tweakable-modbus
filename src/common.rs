use crate::messages::{FunctionCode, ExceptionCode};

use anyhow::{Result, anyhow};

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

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModbusSubprotocol {
    ModbusTCP,
    ModbusRTU,
    ModbusRTUOverTCP,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModbusResult {
    Error(ExceptionCode),
    ReadResult(ModbusDataType),
    WriteConfirmation,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModbusAddress {
    pub table: ModbusTable,
    pub address: u16,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
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


