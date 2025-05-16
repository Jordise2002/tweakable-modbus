mod query;
mod response;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ModbusTable {
    DiscreteInput,
    Coils,
    InputRegisters,
    HoldingRegisters,
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
            ModbusDataType::Register(value) => *value
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
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
}

#[derive(Clone, Copy, PartialEq, Debug)]
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
