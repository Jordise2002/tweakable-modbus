use anyhow::Result;
use std::cell::Cell;
use std::collections::HashMap;

use crate::{
    codec::ModbusSerialize,
    common::{ModbusResult,ModbusAddress, ModbusSubprotocol},
    messages::{ModbusQuery, ModbusResponse},
};

use crate::common::ModbusTable;

//This struct is meant to hold the state of the on going modbus communication
pub struct ModbusContext {
    pub queued_queries: Vec<ModbusQuery>,
    pub on_going_queries: HashMap<u16, ModbusQuery>,
    current_transaction_id: Cell<u16>,
}

impl ModbusContext {
    pub fn new() -> Self {
        ModbusContext {
            current_transaction_id: Cell::new(1),
            queued_queries: Vec::new(),
            on_going_queries: HashMap::new(),
        }
    }

    fn get_next_free_transaction_id(&self) -> u16 {
        let result = self.current_transaction_id.get();
        self.current_transaction_id.set(result + 1);
        result
    }

    pub fn load_queued_queries(&mut self, ammount: u32) {
        self.on_going_queries.clear();

        for _index in 0..ammount {
            if self.queued_queries.is_empty()
            {
                break;
            }
            let query = self.queued_queries.pop().unwrap();
            let message_data = query.get_message_data();
            message_data
                .transaction_id
                .set(Some(self.get_next_free_transaction_id()));
            self.on_going_queries
                .insert(message_data.transaction_id.get().unwrap(), query.clone());
        }
    }

    pub fn serialize_queries(&self, subprotocol: ModbusSubprotocol) -> Result<Vec<u8>> {
        let mut result = vec![];

        for (_transaction_id, query) in &self.on_going_queries {
            result.extend_from_slice(&query.serialize(subprotocol)?);
        }

        Ok(result)
    }

    pub fn process_modbus_responses(
        &mut self,
        responses: Vec<ModbusResponse>,
        address_map: &mut HashMap<ModbusAddress, ModbusResult>,
    ) {
        for response in responses {
            let transaction_id = response.get_message_data().transaction_id.get();
            if let None = transaction_id {
                continue;
            }

            let transaction_id = transaction_id.unwrap();

            if !self.on_going_queries.contains_key(&transaction_id) {
                continue;
            }

            match response {
                ModbusResponse::Error {
                    message_data: _message_data,
                    exception_code,
                } => {
                    let query = self.on_going_queries.get(&transaction_id).unwrap();

                    match query {
                        ModbusQuery::ReadQuery {
                            message_data,
                            params,
                        } => {
                            for address in
                                params.starting_address..params.starting_address + params.ammount
                            {
                                let table = ModbusTable::get_table_from_function_code(
                                    message_data.function_code,
                                )
                                .unwrap();
                                address_map.insert(
                                    ModbusAddress { table, address },
                                    ModbusResult::Error(exception_code),
                                );
                            }
                        }
                        ModbusQuery::SingleWriteQuery {
                            message_data,
                            params,
                        } => {
                            let table = ModbusTable::get_table_from_function_code(
                                message_data.function_code,
                            )
                            .unwrap();
                            address_map.insert(
                                ModbusAddress {
                                    table,
                                    address: params.starting_address,
                                },
                                ModbusResult::Error(exception_code),
                            );
                        }
                        ModbusQuery::MultipleWriteQuery {
                            message_data,
                            params,
                        } => {
                            let table = ModbusTable::get_table_from_function_code(
                                message_data.function_code,
                            )
                            .unwrap();
                            for address in params.starting_address
                                ..params.starting_address + params.values.len() as u16
                            {
                                address_map.insert(
                                    ModbusAddress {
                                        table,
                                        address,
                                    },
                                    ModbusResult::Error(exception_code),
                                );
                            }
                        }
                        ModbusQuery::MultipleReadWriteQuery {
                            message_data,
                            params,
                        } => {
                            let table = ModbusTable::get_table_from_function_code(message_data.function_code).unwrap();
                            for address in params.read_starting_address
                                ..params.read_starting_address + params.read_ammount
                            {
                                address_map.insert(ModbusAddress { table, address}, ModbusResult::Error(exception_code));
                            }

                            for address in params.write_starting_address
                                ..params.write_starting_address + params.values.len() as u16
                            {
                                address_map.insert(ModbusAddress { table, address}, ModbusResult::Error(exception_code));
                            }
                        }
                    }
                }
                ModbusResponse::SingleWriteResponse {
                    message_data,
                    params,
                } => {
                    let table = ModbusTable::get_table_from_function_code(message_data.function_code).unwrap();
                    address_map.insert(ModbusAddress { table, address: params.address}, ModbusResult::WriteConfirmation);
                }
                ModbusResponse::MultipleWriteResponse {
                    message_data,
                    params,
                } => {
                    let table = ModbusTable::get_table_from_function_code(message_data.function_code).unwrap();
                    for address in params.address..params.address + params.ammount {
                        address_map.insert(ModbusAddress { table, address }, ModbusResult::WriteConfirmation);
                    }
                }
                ModbusResponse::ReadResponse {
                    message_data,
                    mut params,
                } => {
                    let query = self.on_going_queries.get(&transaction_id).unwrap();

                    let table = ModbusTable::get_table_from_function_code(message_data.function_code).unwrap();
                    if let ModbusQuery::ReadQuery {
                        message_data: _message_data,
                        params: query_params,
                    } = query
                    {
                        params.values.truncate(query_params.ammount as usize);
                        let mut address = query_params.starting_address;

                        for value in params.values {
                            address_map.insert(ModbusAddress { table, address }, ModbusResult::ReadResult(value));
                            address += 1;
                        }
                    }
                }
            };

            self.on_going_queries.remove(&transaction_id);
        }
    }

    pub fn has_on_going_queries(&self) -> bool {
        return !self.on_going_queries.is_empty();
    }
}
