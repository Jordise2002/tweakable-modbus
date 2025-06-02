use crate::{
    codec::ModbusSerialize,
    common::{ModbusAddress, ModbusDataType, ModbusSubprotocol, SlaveId},
    communication::ModbusSocket,
    messages::{
        response::{self, ReadResponseParameters},
        ExceptionCode, ModbusQuery, ModbusResponse,
    },
    slave::comm::ModbusSlaveCommunicationInfo,
};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::net::SocketAddr;
use tokio::net::TcpStream;

mod comm;

type OnReadFunction = Box<dyn Fn(SlaveId, ModbusAddress) -> Option<ModbusDataType>>;
type OnWriteFunction = Box<dyn Fn(SlaveId, ModbusAddress, ModbusDataType) -> bool>;
pub struct ModbusSlaveConnection {
    comm: ModbusSlaveCommunicationInfo,
    allowed_slaves: HashSet<SlaveId>,
    on_read: OnReadFunction,
    on_write: OnWriteFunction,
}

impl ModbusSlaveConnection {
    pub fn new_tcp(
        &mut self,
        address: SocketAddr,
        on_read: OnReadFunction,
        on_write: OnWriteFunction,
        allowed_slaves: Vec<SlaveId>,
    ) -> Self {
        let comm = ModbusSlaveCommunicationInfo::new_tcp(address);

        let allowed_slaves: HashSet<SlaveId> = allowed_slaves.into_iter().collect();

        ModbusSlaveConnection {
            comm,
            on_read,
            on_write,
            allowed_slaves,
        }
    }

    pub fn handle_query(&self, query: ModbusQuery) -> Result<ModbusResponse> {
        match query {
            ModbusQuery::SingleWriteQuery {
                message_data,
                params,
            } => {
                let address = ModbusAddress {
                    table: params.table,
                    address: params.starting_address,
                };

                let _result = (self.on_write)(message_data.slave_id, address, params.value);

                let params = response::SingleWriteResponseParameters {
                    table: params.table,
                    address: params.starting_address,
                    value: params.value,
                };

                return Ok(ModbusResponse::SingleWriteResponse {
                    message_data,
                    params,
                });
            }
            ModbusQuery::MultipleWriteQuery {
                message_data,
                params,
            } => {
                let mut results = vec![];
                let mut address = ModbusAddress {
                    table: params.table,
                    address: params.starting_address,
                };

                for value in params.values {
                    results.push((self.on_write)(message_data.slave_id, address.clone(), value));
                    address.address += 1;
                }

                let params = response::MultipleWriteResponse {
                    table: params.table,
                    address: params.starting_address,
                    ammount: results.iter().filter(|&&b| b).count() as u16,
                };

                return Ok(ModbusResponse::MultipleWriteResponse {
                    message_data,
                    params,
                });
            }
            ModbusQuery::ReadQuery {
                message_data,
                params,
            } => {
                let mut results = vec![];

                let mut address = ModbusAddress {
                    table: params.table,
                    address: params.starting_address,
                };

                for _index in 0..params.ammount {
                    let result = (self.on_read)(message_data.slave_id, address.clone());

                    if result.is_none() {
                        return Ok(ModbusResponse::Error {
                            message_data,
                            exception_code: ExceptionCode::IllegalDataAddress,
                        });
                    }

                    results.push(result.unwrap());
                    address.address += 1;
                }

                let params = response::ReadResponseParameters {
                    table: params.table,
                    values: results,
                };

                return Ok(ModbusResponse::ReadResponse {
                    message_data,
                    params,
                });
            }
            ModbusQuery::MultipleReadWriteQuery {
                message_data,
                params,
            } => {
                let mut results = vec![];

                let mut write_starting_address = ModbusAddress {
                    table: params.table,
                    address: params.write_starting_address,
                };

                for value in params.values {
                    let _result =
                        (self.on_write)(message_data.slave_id, write_starting_address.clone(), value);
                    write_starting_address.address += 1;
                }

                let mut read_starting_address = ModbusAddress {
                    table: params.table,
                    address: params.read_starting_address,
                };

                for _index in 0..params.read_ammount {
                    let result = (self.on_read)(message_data.slave_id, read_starting_address.clone());

                    if result.is_none() {
                        return Ok(ModbusResponse::Error {
                            message_data,
                            exception_code: ExceptionCode::IllegalDataAddress,
                        });
                    }

                    results.push(result.unwrap());
                    read_starting_address.address += 1;
                }

                let params = ReadResponseParameters {
                    table: params.table,
                    values: results,
                };

                Ok(ModbusResponse::ReadResponse {
                    message_data,
                    params,
                })
            }
        }
    }

    pub async fn handle_connection(&self, mut socket: TcpStream, addr: SocketAddr) -> Result<()> {
        loop {
            let bytes = socket.read().await?;

            let queries =
                crate::messages::ModbusQuery::deserialize(bytes, ModbusSubprotocol::ModbusTCP)?;

            for query in queries {
                if !self
                    .allowed_slaves
                    .contains(&query.get_message_data().slave_id)
                {
                    continue;
                }

                let response = self.handle_query(query)?;
                socket
                    .write(response.serialize(ModbusSubprotocol::ModbusTCP)?)
                    .await;
            }
        }

        Ok(())
    }

    pub async fn serve(&mut self) -> Result<()> {
        if !self.comm.is_bound() {
            self.comm.bind().await?;
        }

        let listener = self.comm.listener.as_ref().unwrap().clone();
        loop {
            let (socket, addr) = listener.accept().await?;

            self.handle_connection(socket, addr).await?;
        }

        Ok(())
    }
}
