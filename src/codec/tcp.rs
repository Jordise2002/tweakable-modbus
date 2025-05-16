use crate::messages::{ModbusDataType, ModbusMessageData, ModbusQuery, ModbusResponse};
use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use std::{mem::discriminant, u16};
pub trait ModbusTcpSerialize
where
    Self: Sized,
{
    fn tcp_serialize(&self) -> Result<Vec<u8>, String>;
    fn tcp_deserialize(data: Vec<u8>) -> Result<Vec<Self>, String>;
}

fn serialize_mbap(message_data: &ModbusMessageData, length: u16) -> Vec<u8> {
    let mut result = Vec::new();

    //Transaction Identifier
    result.extend_from_slice(&message_data.transaction_id.to_be_bytes());

    //Protocol Identifier: 0u16 means Modbus
    result.extend_from_slice(&0u16.to_be_bytes());

    //Length
    result.extend_from_slice(&length.to_be_bytes());

    //Slave Id
    result.push(message_data.slave_id);

    result
}

fn check_same_data_type_variant(values: &Vec<ModbusDataType>) -> bool {
    if let Some((first, others)) = values.split_first() {
        let ref_discriminant = discriminant(first);
        others.iter().all(|e| discriminant(e) == ref_discriminant)
    } else {
        true // Vec vacío = homogéneo por convención
    }
}

fn serialize_values(values: &Vec<ModbusDataType>) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();

    if !check_same_data_type_variant(values) {
        return Err(String::from(
            "All values in a query must have the same type",
        ));
    }

    if values.is_empty() {
        return Err(String::from("At least one value must be sent"));
    }

    let first_value = values.first().unwrap();

    match first_value {
        ModbusDataType::Coil(_) => {
            let length = if values.len() % 8 == 0 {
                values.len() as u8 / 8
            } else {
                values.len() as u8 / 8 + 1
            };

            result.push(length);

            let mut counter = 0;
            let mut aux_byte = 0;

            for value in values {
                if let ModbusDataType::Coil(value) = value {
                    if *value {
                        aux_byte = aux_byte << 1;
                    } else {
                        aux_byte = aux_byte << 0;
                    }

                    counter += 1;

                    if counter == 8 {
                        result.push(aux_byte);
                        aux_byte = 0;
                        counter = 0;
                    }
                }
            }
        }
        ModbusDataType::Register(_) => {
            let length = values.len() as u8 * 2;

            result.push(length);

            for value in values {
                if let ModbusDataType::Register(value) = value {
                    result.extend_from_slice(&value.to_be_bytes());
                }
            }
        }
    }
    Ok(result)
}

impl ModbusTcpSerialize for ModbusQuery {
    fn tcp_deserialize(data: Vec<u8>) -> Result<Vec<Self>, String> {
        let result = Vec::new();

        Ok(result)
    }

    fn tcp_serialize(&self) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();

        match self {
            ModbusQuery::ReadQuery {
                message_data,
                table: _table,
                starting_address,
                ammount,
            } => {
                let mut read_query = Vec::new();

                //Function code
                read_query.push(message_data.function_code as u8);

                //Starting Address
                read_query.extend_from_slice(&starting_address.to_be_bytes());

                //Ammount
                read_query.extend_from_slice(&ammount.to_be_bytes());

                let mbap = serialize_mbap(message_data, (read_query.len() + 1) as u16);

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&read_query);
            }
            ModbusQuery::SingleWriteQuery {
                message_data,
                table: _table,
                address,
                value,
            } => {
                let mut single_write_query = Vec::new();

                //Function code
                single_write_query.push(message_data.function_code as u8);

                //Address
                single_write_query.extend_from_slice(&address.to_be_bytes());

                //Value
                let value = value.get_representation();
                single_write_query.extend_from_slice(&value.to_be_bytes());

                let mbap = serialize_mbap(message_data, (single_write_query.len() + 1) as u16);

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&single_write_query);
            }
            ModbusQuery::MultipleWriteQuery {
                message_data,
                table: _table,
                starting_address,
                values,
            } => {
                let mut multiple_write_query = Vec::new();

                //Function code
                multiple_write_query.push(message_data.function_code as u8);

                //Starting Address
                multiple_write_query.extend_from_slice(&starting_address.to_be_bytes());

                //Ammount
                let ammount = values.len() as u16;
                multiple_write_query.extend_from_slice(&ammount.to_be_bytes());

                //Values
                let values = serialize_values(values)?;
                multiple_write_query.extend_from_slice(&values);

                let mbap = serialize_mbap(message_data, (multiple_write_query.len() + 1) as u16);

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&multiple_write_query);
            }
            ModbusQuery::MultipleReadWriteQuery {
                message_data,
                table: _table,
                read_starting_address,
                read_ammount,
                write_starting_address,
                values,
            } => {
                let mut multiple_write_read_query = Vec::new();

                //Function code
                multiple_write_read_query.push(message_data.function_code as u8);

                //Read Starting Address
                multiple_write_read_query.extend_from_slice(&read_starting_address.to_be_bytes());

                //Read Ammount
                multiple_write_read_query.extend_from_slice(&read_ammount.to_be_bytes());

                //Write Starting Address
                multiple_write_read_query.extend_from_slice(&write_starting_address.to_be_bytes());

                //Write Ammount
                let write_ammount = values.len() as u16;
                multiple_write_read_query.extend_from_slice(&write_ammount.to_be_bytes());

                //Write values
                let values = serialize_values(values)?;
                multiple_write_read_query.extend_from_slice(&values);

                let mbap = serialize_mbap(message_data, multiple_write_read_query.len() as u16 + 1);

                result.extend_from_slice(&mbap);
                result.extend_from_slice(&multiple_write_read_query);
            }
        };
        Ok(result)
    }
}
