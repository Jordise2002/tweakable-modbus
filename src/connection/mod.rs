use crate::messages::{ExceptionCode, FunctionCode, ModbusDataType, ModbusMessageData, ModbusQuery, ModbusResponse, ModbusTable};
use crate::codec::ModbusSerialize;
use communication::ModbusCommunicationInfo;
use context::ModbusContext;

use anyhow::{anyhow, Result};
use std::{cell::Cell, collections::HashMap, net::SocketAddr};
use tokio::time::{sleep, Duration};

mod communication;
mod context;
mod socket;

const MAX_MODBUS_RESPONSE_TIME: Duration = tokio::time::Duration::from_millis(5000);

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
    WriteConfirmation
}

#[derive(Clone,Debug, PartialEq, Eq, Hash)]
pub struct ModbusAddress {
    pub table: ModbusTable,
    pub address: u16
}

pub struct ModbusConnection {
    comm: ModbusCommunicationInfo,
    context: ModbusContext,
    subprotocol: ModbusSubprotocol,
}

impl ModbusConnection {
    pub fn new_tcp(address: SocketAddr) -> Self {
        let comm = ModbusCommunicationInfo::new_tcp(address);

        let context = ModbusContext::new();

        ModbusConnection {
            comm,
            context,
            subprotocol: ModbusSubprotocol::ModbusTCP,
        }
    }

    pub async fn query_with_max_time(
        &mut self,
        max_response_time: Duration,
    ) -> Result<HashMap<ModbusAddress, ModbusResult>> {
        self.context.load_queued_queries();

        let all_queries = self.context.serialize_all_queries(self.subprotocol)?;

        if !self.comm.is_connected().await {
            self.comm.connect().await?;
        }

        let comm = self
            .comm
            .comm
            .as_mut()
            .ok_or_else(|| anyhow!("Socket wasn't intialised!"))?;

        comm.write(all_queries).await?;

        let mut results = HashMap::new();
        let time_out = sleep(max_response_time);
        tokio::pin!(time_out);

        let mut stop_listening = false;
        loop {
            tokio::select! {
                bytes = comm.read() => {
                    if let Err(_) = bytes {
                        break;
                    }

                    let bytes = bytes.unwrap();

                    let responses = ModbusResponse::deserialize(bytes, self.subprotocol)?;

                    self.context.process_modbus_responses(responses, & mut results);
                    
                    if ! self.context.has_on_going_queries()
                    {
                        stop_listening = true;
                    }

                }
                _ = & mut time_out => {
                    stop_listening = true;
                }
            };

            if stop_listening {
                break;
            }
        }

        Ok(results)
    }

    pub fn query(
        &mut self,
    ) -> impl std::future::Future<Output = Result<HashMap<ModbusAddress, ModbusResult>>> + '_ {
        self.query_with_max_time(MAX_MODBUS_RESPONSE_TIME)
    }

    fn add_read_query(
        &mut self,
        slave_id: u8,
        address: u16,
        ammount: u16,
        function_code: FunctionCode,
    ) -> Result<()> {
        let message_data = ModbusMessageData {
            slave_id,
            function_code,
            transaction_id: Cell::new(None),
        };

        let table = ModbusTable::get_table_from_function_code(function_code)
            .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

        let params = crate::messages::query::ReadQueryParameters {
            starting_address: address,
            ammount,
            table,
        };

        let query = ModbusQuery::ReadQuery {
            message_data,
            params,
        };

        self.context.queued_queries.push(query);

        Ok(())
    }

    fn add_single_write_query(
        &mut self,
        slave_id: u8,
        address: u16,
        value: ModbusDataType,
        function_code: FunctionCode,
    ) -> Result<()> {
        let message_data = ModbusMessageData {
            slave_id,
            function_code,
            transaction_id: Cell::new(None),
        };

        let table = ModbusTable::get_table_from_function_code(function_code)
            .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

        let params = crate::messages::query::SingleWriteQueryParameters {
            starting_address: address,
            table,
            value,
        };

        let query = ModbusQuery::SingleWriteQuery {
            message_data,
            params,
        };

        self.context.queued_queries.push(query);

        Ok(())
    }

    fn add_multiple_write_query(
        &mut self,
        slave_id: u8,
        address: u16,
        values: Vec<ModbusDataType>,
        function_code: FunctionCode,
    ) -> Result<()> {
        let message_data = ModbusMessageData {
            slave_id,
            function_code,
            transaction_id: Cell::new(None),
        };

        let table = ModbusTable::get_table_from_function_code(function_code)
            .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

        let params = crate::messages::query::MultipleWriteQueryParameters {
            table,
            starting_address: address,
            values,
        };

        let query = ModbusQuery::MultipleWriteQuery {
            message_data,
            params,
        };

        self.context.queued_queries.push(query);

        Ok(())
    }

    fn add_multiple_read_write_query(
        &mut self,
        slave_id: u8,
        read_starting_address: u16,
        read_ammount: u16,
        write_starting_address: u16,
        values: Vec<ModbusDataType>,
        function_code: FunctionCode,
    ) -> Result<()> {
        let message_data = ModbusMessageData {
            function_code,
            slave_id,
            transaction_id: Cell::new(None),
        };

        let table = ModbusTable::get_table_from_function_code(function_code)
            .ok_or_else(|| anyhow!("Function code doesn't address any table"))?;

        let params = crate::messages::query::MultipleReadWriteQueryParameters {
            table,
            read_starting_address,
            read_ammount,
            write_starting_address,
            values,
        };

        let query = ModbusQuery::MultipleReadWriteQuery {
            message_data,
            params,
        };

        self.context.queued_queries.push(query);

        Ok(())
    }

    pub fn add_multiple_read_write_holding_registers_query(
        &mut self,
        slave_id: u8,
        read_starting_address: u16,
        read_ammount: u16,
        write_starting_address: u16,
        values: Vec<u16>,
    ) -> Result<()> {
        let mut modbus_values = vec![];

        for value in values {
            modbus_values.push(ModbusDataType::Register(value));
        }

        self.add_multiple_read_write_query(
            slave_id,
            read_starting_address,
            read_ammount,
            write_starting_address,
            modbus_values,
            FunctionCode::ReadWriteMultipleRegisters,
        )
    }

    pub fn add_write_multiple_coils_query(
        &mut self,
        slave_id: u8,
        address: u16,
        values: Vec<bool>,
    ) -> Result<()> {
        let mut modbus_values = vec![];

        for value in values {
            modbus_values.push(ModbusDataType::Coil(value));
        }

        self.add_multiple_write_query(
            slave_id,
            address,
            modbus_values,
            FunctionCode::WriteMultipleCoils,
        )
    }

    pub fn add_write_multiple_holding_registers_query(
        &mut self,
        slave_id: u8,
        address: u16,
        values: Vec<u16>,
    ) -> Result<()> {
        let mut modbus_values = vec![];
        for value in values {
            modbus_values.push(ModbusDataType::Register(value));
        }
        self.add_multiple_write_query(
            slave_id,
            address,
            modbus_values,
            FunctionCode::WriteMultipleHoldingRegisters,
        )
    }

    pub fn add_write_coil_query(&mut self, slave_id: u8, address: u16, value: bool) -> Result<()> {
        self.add_single_write_query(
            slave_id,
            address,
            ModbusDataType::Coil(value),
            FunctionCode::WriteSingleCoil,
        )
    }

    pub fn add_write_holding_register_query(
        &mut self,
        slave_id: u8,
        address: u16,
        value: u16,
    ) -> Result<()> {
        self.add_single_write_query(
            slave_id,
            address,
            ModbusDataType::Register(value),
            FunctionCode::WriteSingleHoldingRegister,
        )
    }

    pub fn add_read_coils_query(&mut self, slave_id: u8, address: u16, ammount: u16) -> Result<()> {
        self.add_read_query(slave_id, address, ammount, FunctionCode::ReadCoils)
    }

    pub fn add_read_holding_registers_query(
        &mut self,
        slave_id: u8,
        address: u16,
        ammount: u16,
    ) -> Result<()> {
        self.add_read_query(
            slave_id,
            address,
            ammount,
            FunctionCode::ReadMultipleHoldingRegister,
        )
    }

    pub fn add_read_discrete_inputs_query(
        &mut self,
        slave_id: u8,
        address: u16,
        ammount: u16,
    ) -> Result<()> {
        self.add_read_query(slave_id, address, ammount, FunctionCode::ReadDiscreteInputs)
    }

    pub fn add_read_input_registers_query(
        &mut self,
        slave_id: u8,
        address: u16,
        ammount: u16,
    ) -> Result<()> {
        self.add_read_query(slave_id, address, ammount, FunctionCode::ReadInputRegisters)
    }
}
