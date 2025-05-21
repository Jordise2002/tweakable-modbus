use std::{fs::read, io::Read, os::linux::raw};

use crate::codec::tcp::ModbusTcpSerialize;

use super::*;
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};

fn deserialize_error_response(
    message_data: ModbusMessageData,
    mut data: std::io::Cursor<Vec<u8>>,
) -> Result<ModbusResponse> {
    let exception_code = ExceptionCode::try_from(data.read_u8()?)?;

    Ok(ModbusResponse::Error {
        message_data,
        exception_code,
    })
}

fn deserialize_single_write_response(
    message_data: ModbusMessageData,
    mut data: std::io::Cursor<Vec<u8>>,
) -> Result<ModbusResponse> {
    let table = ModbusTable::get_table_from_function_code(FunctionCode::try_from(data.read_u8()?)?)
        .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

    let address = data.read_u16::<BigEndian>()?;

    let raw_value = data.read_u16::<BigEndian>()?;

    let value = match table {
        ModbusTable::Coils | ModbusTable::DiscreteInput => {
            ModbusDataType::coil_from_representation(raw_value)?
        }
        ModbusTable::InputRegisters | ModbusTable::HoldingRegisters => {
            ModbusDataType::Register(raw_value)
        }
    };

    let params = SingleWriteResponseParameters {
        table,
        address,
        value,
    };

    Ok(ModbusResponse::SingleWriteResponse {
        message_data,
        params,
    })
}

fn deserialize_read_response(
    message_data: ModbusMessageData,
    mut data: std::io::Cursor<Vec<u8>>,
) -> Result<ModbusResponse> {
    let table = ModbusTable::get_table_from_function_code(FunctionCode::try_from(data.read_u8()?)?)
        .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

    let values = crate::codec::utils::deserialize_values(table, None, &mut data)?;

    let params = ReadResponseParameters { table, values };

    Ok(ModbusResponse::ReadResponse {
        message_data,
        params,
    })
}

fn deserialize_multiple_write_response(
    message_data: ModbusMessageData,
    mut data: std::io::Cursor<Vec<u8>>,
) -> Result<ModbusResponse> {
    let table = ModbusTable::get_table_from_function_code(FunctionCode::try_from(data.read_u8()?)?)
        .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

    let address = data.read_u16::<BigEndian>()?;

    let ammount = data.read_u16::<BigEndian>()?;

    let params = MultipleWriteResponse {
        table,
        address,
        ammount,
    };

    Ok(ModbusResponse::MultipleWriteResponse {
        message_data,
        params,
    })
}

impl ModbusTcpSerialize for ModbusResponse {
    fn tcp_deserialize(data: Vec<u8>) -> Result<Vec<Self>> {
        let mut result = vec![];

        let mut data = std::io::Cursor::new(data);

        let mut size_left = data.get_ref().len();

        while size_left > 0 {
            let (mut message_data, length) = crate::codec::tcp::deserialize_mbap(&mut data)?;

            let mut message_body = vec![0u8; length as usize];

            data.read_exact(&mut message_body)?;

            let message_body = std::io::Cursor::new(message_body);

            let raw_function_code = data.read_u8()?;

            //Is in error range
            if raw_function_code > 0x80 {
                message_data.function_code = FunctionCode::try_from(raw_function_code - 0x80)?;
                if let Ok(response) = deserialize_error_response(message_data, message_body) {
                    result.push(response);
                }
                continue;
            } else {
                message_data.function_code = FunctionCode::try_from(raw_function_code)?;

                match message_data.function_code {
                    FunctionCode::WriteSingleCoil | FunctionCode::WriteSingleHoldingRegister => {
                        if let Ok(response) =
                            deserialize_single_write_response(message_data, message_body)
                        {
                            result.push(response);
                        }
                    }
                    FunctionCode::ReadCoils
                    | FunctionCode::ReadDiscreteInputs
                    | FunctionCode::ReadMultipleHoldingRegister
                    | FunctionCode::ReadInputRegisters
                    | FunctionCode::ReadWriteMultipleRegisters => {
                        if let Ok(response) = deserialize_read_response(message_data, message_body)
                        {
                            result.push(response);
                        }
                    }
                    FunctionCode::WriteMultipleCoils
                    | FunctionCode::WriteMultipleHoldingRegisters => {
                        if let Ok(response) =
                            deserialize_multiple_write_response(message_data, message_body)
                        {
                            result.push(response);
                        }
                    }
                    _ => {}
                }
            }
            size_left = data.get_ref().len() - data.position() as usize;
        }

        Ok(result)
    }

    fn tcp_serialize(&self) -> Result<Vec<u8>> {
        let mut result = vec![];
        match self
        {
            ModbusResponse::ReadResponse { message_data, params } => {
                let mut read_response = vec![];

                read_response.push(message_data.function_code as u8);

                let values = crate::codec::utils::serialize_values(&params.values)?;

                read_response.extend_from_slice(&values);

                let mbap = crate::codec::tcp::serialize_mbap(message_data, read_response.len() as u16 + 1);

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&read_response);
            }
            ModbusResponse::SingleWriteResponse { message_data , params } => {
                let mut single_write_response = vec![];

                single_write_response.push(message_data.function_code as u8);

                single_write_response.extend_from_slice(&params.address.to_be_bytes());

                let value = params.value.get_representation();

                single_write_response.extend_from_slice(&value.to_be_bytes());

                let mbap = crate::codec::tcp::serialize_mbap(message_data, single_write_response.len() as u16 + 1);

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&single_write_response);
            }
            ModbusResponse::MultipleWriteResponse { message_data, params } => {
                let mut multiple_write_response = vec![];

                multiple_write_response.push(message_data.function_code as u8);

                multiple_write_response.extend_from_slice(&params.address.to_be_bytes());

                multiple_write_response.extend_from_slice(&params.ammount.to_be_bytes());

                let mbap = crate::codec::tcp::serialize_mbap(message_data, multiple_write_response.len() as u16 + 1);

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&multiple_write_response);
            }
            ModbusResponse::Error { message_data, exception_code } => {
                let mut error_response = vec![];

                error_response.push(message_data.function_code as u8 + 0x80);

                error_response.push(*exception_code as u8);
                
                let mbap = crate::codec::tcp::serialize_mbap(message_data, error_response.len() as u16 + 1);

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&error_response);
            }
            _ => {

            }
        };
        Ok(result)
    }
}
