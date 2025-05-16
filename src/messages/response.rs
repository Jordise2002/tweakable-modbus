use super::*;

#[derive(Clone,PartialEq,Debug)]
pub enum ModbusResponse 
{
    ReadResponse{message_data: ModbusMessageData, values: Vec<ModbusDataType>},
    SingleWriteResponse{message_data: ModbusMessageData, address: u16, value: ModbusDataType},
    MultipleWriteResponse{message_data: ModbusMessageData, address: u16, ammount: u16},

    Error{message_data : ModbusMessageData, exception_code: ExceptionCode}
}