use super::*;
use crate::codec::tcp::ModbusTcpSerialize;

use anyhow::{anyhow, Result};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

impl ModbusTcpSerialize for ModbusQuery {
    fn tcp_deserialize(data: Vec<u8>) -> Result<Vec<Self>> {
        let mut result = Vec::new();

        let mut data = Cursor::new(data);

        let mut size_left = data.get_ref().len();

        while size_left > 0 {
            let (mut message_data, length) = crate::codec::tcp::deserialize_mbap(&mut data)?;

            let mut message_body = vec![0u8; length as usize];

            data.read_exact(&mut message_body)?;

            let mut message_body = Cursor::new(message_body);

            message_data.function_code = FunctionCode::try_from(message_body.read_u8()?)?;

            let query = match message_data.function_code {
                FunctionCode::ReadCoils
                | FunctionCode::ReadDiscreteInputs
                | FunctionCode::ReadInputRegisters
                | FunctionCode::ReadMultipleHoldingRegister => {
                    if let Ok(params) = deserialize_read_query(&message_data, message_body) {
                        ModbusQuery::ReadQuery {
                            message_data,
                            params,
                        }
                    } else {
                        continue;
                    }
                }
                FunctionCode::WriteSingleCoil | FunctionCode::WriteSingleHoldingRegister => {
                    if let Ok(params) = deserialize_single_write_query(&message_data, message_body)
                    {
                        ModbusQuery::SingleWriteQuery {
                            message_data,
                            params,
                        }
                    } else {
                        continue;
                    }
                }
                FunctionCode::WriteMultipleCoils | FunctionCode::WriteMultipleHoldingRegisters => {
                    if let Ok(params) =
                        deserialize_multiple_write_query(&message_data, message_body)
                    {
                        ModbusQuery::MultipleWriteQuery {
                            message_data,
                            params,
                        }
                    } else {
                        continue;
                    }
                }
                FunctionCode::ReadWriteMultipleRegisters => {
                    if let Ok(params) =
                        deserialize_multiple_read_write_query(&message_data, message_body)
                    {
                        ModbusQuery::MultipleReadWriteQuery {
                            message_data,
                            params,
                        }
                    } else {
                        continue;
                    }
                }
                _ => {
                    continue;
                }
            };

            result.push(query);

            size_left = data.get_ref().len() - data.position() as usize;
        }
        Ok(result)
    }

    fn tcp_serialize(&self) -> Result<Vec<u8>> {
        let mut result = Vec::new();

        match self {
            ModbusQuery::ReadQuery {
                message_data,
                params,
            } => {
                let mut read_query = Vec::new();

                //Function code
                read_query.push(message_data.function_code as u8);

                //Starting Address
                read_query.extend_from_slice(&params.starting_address.to_be_bytes());

                //Ammount
                read_query.extend_from_slice(&params.ammount.to_be_bytes());

                let mbap =
                    crate::codec::tcp::serialize_mbap(message_data, (read_query.len() + 1) as u16)?;

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&read_query);
            }
            ModbusQuery::SingleWriteQuery {
                message_data,
                params,
            } => {
                let mut single_write_query = Vec::new();

                //Function code
                single_write_query.push(message_data.function_code as u8);

                //Address
                single_write_query.extend_from_slice(&params.starting_address.to_be_bytes());

                //Value
                let value = params.value.get_representation();
                single_write_query.extend_from_slice(&value.to_be_bytes());

                let mbap = crate::codec::tcp::serialize_mbap(
                    message_data,
                    (single_write_query.len() + 1) as u16,
                )?;

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&single_write_query);
            }
            ModbusQuery::MultipleWriteQuery {
                message_data,
                params,
            } => {
                let mut multiple_write_query = Vec::new();

                //Function code
                multiple_write_query.push(message_data.function_code as u8);

                //Starting Address
                multiple_write_query.extend_from_slice(&params.starting_address.to_be_bytes());

                //Ammount
                let ammount = params.values.len() as u16;
                multiple_write_query.extend_from_slice(&ammount.to_be_bytes());

                //Values
                let values = crate::codec::utils::serialize_values(&params.values)?;
                multiple_write_query.extend_from_slice(&values);

                let mbap = crate::codec::tcp::serialize_mbap(
                    message_data,
                    (multiple_write_query.len() + 1) as u16,
                )?;

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&multiple_write_query);
            }
            ModbusQuery::MultipleReadWriteQuery {
                message_data,
                params,
            } => {
                let mut multiple_write_read_query = Vec::new();

                //Function code
                multiple_write_read_query.push(message_data.function_code as u8);

                //Read Starting Address
                multiple_write_read_query
                    .extend_from_slice(&params.read_starting_address.to_be_bytes());

                //Read Ammount
                multiple_write_read_query.extend_from_slice(&params.read_ammount.to_be_bytes());

                //Write Starting Address
                multiple_write_read_query
                    .extend_from_slice(&params.write_starting_address.to_be_bytes());

                //Write Ammount
                let write_ammount = params.values.len() as u16;
                multiple_write_read_query.extend_from_slice(&write_ammount.to_be_bytes());

                //Write values
                let values = crate::codec::utils::serialize_values(&params.values)?;
                multiple_write_read_query.extend_from_slice(&values);

                let mbap = crate::codec::tcp::serialize_mbap(
                    message_data,
                    multiple_write_read_query.len() as u16 + 1,
                )?;

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&multiple_write_read_query);
            }
        };
        Ok(result)
    }
}

fn deserialize_read_query(
    message_data: &ModbusMessageData,
    mut data: Cursor<Vec<u8>>,
) -> Result<ReadQueryParameters> {
    let table = ModbusTable::get_table_from_function_code(message_data.function_code)
        .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

    let starting_address = data.read_u16::<BigEndian>()?;

    let ammount = data.read_u16::<BigEndian>()?;

    let position = data.position() as usize;

    let len = data.get_ref().len();

    if position != len {
        return Err(anyhow!(
            "Read query too long, {} too many bytes",
            len - position
        ));
    }

    Ok(ReadQueryParameters {
        table,
        starting_address,
        ammount,
    })
}

fn deserialize_single_write_query(
    message_data: &ModbusMessageData,
    mut data: Cursor<Vec<u8>>,
) -> Result<SingleWriteQueryParameters> {
    let table = ModbusTable::get_table_from_function_code(message_data.function_code)
        .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

    let starting_address = data.read_u16::<BigEndian>()?;

    let raw_value = data.read_u16::<BigEndian>()?;

    let position = data.position() as usize;

    let len = data.get_ref().len();

    if position != len {
        return Err(anyhow!(
            "Single Write query too long, {} too many bytes",
            len - position
        ));
    }

    let value = match table {
        ModbusTable::Coils | ModbusTable::DiscreteInput => {
            ModbusDataType::coil_from_representation(raw_value)?
        }
        ModbusTable::HoldingRegisters | ModbusTable::InputRegisters => {
            ModbusDataType::Register(raw_value)
        }
    };

    Ok(SingleWriteQueryParameters {
        table,
        starting_address,
        value,
    })
}

fn deserialize_multiple_write_query(
    message_data: &ModbusMessageData,
    mut data: Cursor<Vec<u8>>,
) -> Result<MultipleWriteQueryParameters> {
    let table = ModbusTable::get_table_from_function_code(message_data.function_code)
        .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

    let starting_address = data.read_u16::<BigEndian>()?;

    let ammount = data.read_u16::<BigEndian>()?;

    let values = crate::codec::utils::deserialize_values(table, Some(ammount), &mut data)?;

    let position = data.position() as usize;

    let len = data.get_ref().len();

    if position != len {
        return Err(anyhow!(
            "Multiple Write query too long, {} too many bytes",
            len - position
        ));
    }

    Ok(MultipleWriteQueryParameters {
        table,
        starting_address,
        values,
    })
}

fn deserialize_multiple_read_write_query(
    message_data: &ModbusMessageData,
    mut data: Cursor<Vec<u8>>,
) -> Result<MultipleReadWriteQueryParameters> {
    let table = ModbusTable::get_table_from_function_code(message_data.function_code)
        .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

    let read_starting_address = data.read_u16::<BigEndian>()?;

    let read_ammount = data.read_u16::<BigEndian>()?;

    let write_starting_address = data.read_u16::<BigEndian>()?;

    let write_ammount = data.read_u16::<BigEndian>()?;

    let values = crate::codec::utils::deserialize_values(table, Some(write_ammount), &mut data)?;

    let position = data.position() as usize;

    let len = data.get_ref().len();

    if position != len {
        return Err(anyhow!(
            "Multiple Write query too long, {} too many bytes",
            len - position
        ));
    }

    Ok(MultipleReadWriteQueryParameters {
        table,
        read_starting_address,
        read_ammount,
        write_starting_address,
        values,
    })
}

#[cfg(test)]
mod test {
    use crate::connection::ModbusSubprotocol;

    use super::*;

    fn test_queries_serialization(input: Vec<ModbusQuery>) {
        let mut bytes = vec![];
        for input in input.clone() {
            let query_bytes = input.serialize(ModbusSubprotocol::ModbusTCP).unwrap();
            bytes.extend_from_slice(&query_bytes);
        }

        let output = ModbusQuery::deserialize(bytes, ModbusSubprotocol::ModbusTCP).unwrap();

        println!("{:?}", input);
        assert_eq!(input, output);
    }
    #[test]
    fn test_serialization_deserialization_read_query() {
        let input = vec![
            ModbusQuery::ReadQuery {
                message_data: ModbusMessageData {
                    slave_id: 1,
                    function_code: FunctionCode::ReadCoils,
                    transaction_id: Cell::new(Some(1)),
                },
                params: query::ReadQueryParameters {
                    table: ModbusTable::Coils,
                    starting_address: 0x00FF,
                    ammount: 32,
                },
            },
            ModbusQuery::ReadQuery {
                message_data: ModbusMessageData {
                    slave_id: 3,
                    function_code: FunctionCode::ReadInputRegisters,
                    transaction_id: Cell::new(Some(2)),
                },
                params: query::ReadQueryParameters {
                    table: ModbusTable::InputRegisters,
                    starting_address: 0x00,
                    ammount: 17,
                },
            },
            ModbusQuery::ReadQuery {
                message_data: ModbusMessageData {
                    slave_id: 1,
                    function_code: FunctionCode::ReadMultipleHoldingRegister,
                    transaction_id: Cell::new(Some(1),)
                },
                params: query::ReadQueryParameters {
                    table: ModbusTable::HoldingRegisters,
                    starting_address: 0x003,
                    ammount: 90,
                },
            },
            ModbusQuery::ReadQuery {
                message_data: ModbusMessageData {
                    slave_id: 0xFF,
                    function_code: FunctionCode::ReadDiscreteInputs,
                    transaction_id: Cell::new(Some(2)),
                },
                params: query::ReadQueryParameters {
                    table: ModbusTable::DiscreteInput,
                    starting_address: 1,
                    ammount: 0,
                },
            },
        ];

        test_queries_serialization(input);
    }

    #[test]
    fn test_serialization_deserialization_single_write_query() {
        let input = vec![
            ModbusQuery::SingleWriteQuery {
                message_data: ModbusMessageData {
                    slave_id: 3,
                    function_code: FunctionCode::WriteSingleCoil,
                    transaction_id: Cell::new(Some(33)),
                },
                params: query::SingleWriteQueryParameters {
                    table: ModbusTable::Coils,
                    starting_address: 38,
                    value: ModbusDataType::Coil(false),
                },
            },
            ModbusQuery::SingleWriteQuery {
                message_data: ModbusMessageData {
                    slave_id: 90,
                    function_code: FunctionCode::WriteSingleHoldingRegister,
                    transaction_id: Cell::new(Some(87)),
                },
                params: query::SingleWriteQueryParameters {
                    table: ModbusTable::HoldingRegisters,
                    starting_address: 67,
                    value: ModbusDataType::Register(33),
                },
            },
        ];
        test_queries_serialization(input);
    }

    #[test]
    fn test_serialization_deserialization_multiple_write_query() {
        let input = vec![
            ModbusQuery::MultipleWriteQuery {
                message_data: ModbusMessageData {
                    slave_id: 8,
                    function_code: FunctionCode::WriteMultipleCoils,
                    transaction_id: Cell::new(Some(67)),
                },
                params: query::MultipleWriteQueryParameters {
                    table: ModbusTable::Coils,
                    starting_address: 0x33,
                    values: vec![
                        ModbusDataType::Coil(false),
                        ModbusDataType::Coil(false),
                        ModbusDataType::Coil(true),
                    ],
                },
            },
            ModbusQuery::MultipleWriteQuery {
                message_data: ModbusMessageData {
                    slave_id: 33,
                    function_code: FunctionCode::WriteMultipleHoldingRegisters,
                    transaction_id: Cell::new(Some(89)),
                },
                params: query::MultipleWriteQueryParameters {
                    table: ModbusTable::HoldingRegisters,
                    starting_address: 032,
                    values: vec![
                        ModbusDataType::Register(10),
                        ModbusDataType::Register(32),
                        ModbusDataType::Register(33),
                    ],
                },
            },
        ];
        test_queries_serialization(input);
    }

    #[test]
    fn test_serialization_deserialization_multiple_read_write_queries() {
        let input = vec![ModbusQuery::MultipleReadWriteQuery {
            message_data: ModbusMessageData {
                slave_id: 33,
                function_code: FunctionCode::ReadWriteMultipleRegisters,
                transaction_id: Cell::new(Some(83)),
            },
            params: query::MultipleReadWriteQueryParameters{
                read_starting_address: 0,
                read_ammount: 300,
                table: ModbusTable::HoldingRegisters,
                write_starting_address: 0,
                values: vec![ModbusDataType::Register(33),
                ModbusDataType::Register(22)]
            },
        }];
        test_queries_serialization(input);
    }
}
