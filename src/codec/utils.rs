use crate::messages::{ModbusDataType, ModbusTable};

use anyhow::{anyhow, Result};
use tokio::time::error::Elapsed;
use std::mem::discriminant;
use std::io::{Cursor};
use byteorder::{BigEndian, ReadBytesExt};

pub fn serialize_values(values: &Vec<ModbusDataType>) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    if !check_same_data_type_variant(values) {
        return Err(anyhow!("All values in a query must have the same type",));
    }

    if values.is_empty() {
        return Err(anyhow!("At least one value must be sent"));
    }

    let first_value = values.first().ok_or_else(|| anyhow!("No first value"))?;

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
                    aux_byte = aux_byte << 1;
                    if *value {
                        aux_byte = aux_byte | 0b1;
                    }

                    counter += 1;

                    if counter == 8 {
                        result.push(aux_byte);
                        aux_byte = 0;
                        counter = 0;
                    }
                }
            }
            if counter != 0 {
                result.push(aux_byte)
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

pub fn deserialize_values(
    table: ModbusTable,
    ammount: Option<u16>,
    data: &mut Cursor<Vec<u8>>,
) -> Result<Vec<ModbusDataType>> {

    let byte_count = data.read_u8()? as u16;

    //If we have no ammount, we make one up
    //I have to do this because modbus doesn't tell you how many registers como in a read response
    let ammount = if let Some(ammount) = ammount {
        ammount
    }
    else {
        match table {
            ModbusTable::Coils | ModbusTable::DiscreteInput => byte_count * 8,
            ModbusTable::HoldingRegisters | ModbusTable::InputRegisters => byte_count / 2
        } 
    };

    let expected_byte_count = match table {
        ModbusTable::Coils | ModbusTable::DiscreteInput => {
            if ammount % 8 == 0 {
                ammount / 8
            } else {
                (ammount / 8) + 1
            }
        }
        ModbusTable::InputRegisters | ModbusTable::HoldingRegisters => ammount * 2,
    };

    if expected_byte_count != byte_count {
        return Err(anyhow!(
            "Expected {} bytes for values, got {}",
            expected_byte_count,
            byte_count
        ));
    }

    let mut values = vec![];
    match table
    {
        ModbusTable::Coils | ModbusTable::DiscreteInput => {
            let mut counter = 0;
            let mut aux_byte = data.read_u8()?;
            for _ in 0..ammount {
                if counter == 8
                {
                    aux_byte = data.read_u8()?;
                    counter = 0;
                }
                let value = (aux_byte & 0b1) != 0;
                values.push(ModbusDataType::Coil(value));
                aux_byte = aux_byte >> 1;
                counter += 1;
            }
            values.reverse();
        },
        ModbusTable::HoldingRegisters | ModbusTable::InputRegisters => {
            for _ in 0..ammount {
                let raw_value = data.read_u16::<BigEndian>()?;
                values.push(ModbusDataType::Register(raw_value));
            }
        }
    }


    Ok(values)
}

fn check_same_data_type_variant(values: &Vec<ModbusDataType>) -> bool {
    if let Some((first, others)) = values.split_first() {
        let ref_discriminant = discriminant(first);
        others.iter().all(|e| discriminant(e) == ref_discriminant)
    } else {
        true // Vec vacío = homogéneo por convención
    }
}