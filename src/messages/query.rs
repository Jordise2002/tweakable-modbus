use super::*;

#[derive(Clone,PartialEq,Debug)]
pub enum ModbusQuery
{
    ReadQuery {message_data: ModbusMessageData, table: ModbusTable, starting_address: u16, ammount: u16},
    SingleWriteQuery {message_data: ModbusMessageData, table: ModbusTable, address: u16, value: ModbusDataType},
    MultipleWriteQuery {message_data: ModbusMessageData, table: ModbusTable,  starting_address:u16, values: Vec<ModbusDataType>},
    MultipleReadWriteQuery{message_data: ModbusMessageData, table: ModbusTable, read_starting_address: u16, read_ammount: u16, write_starting_address: u16, values: Vec<ModbusDataType>}
}