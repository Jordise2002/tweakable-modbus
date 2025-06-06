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
use anyhow::Result;
use std::net::{IpAddr, SocketAddr};
use std::{collections::HashSet, time::Duration};
use tokio::net::TcpStream;

mod comm;

type OnReadFunction = Box<
    dyn Fn(SlaveId, ModbusAddress) -> std::result::Result<ModbusDataType, ExceptionCode>
        + Send
        + Sync,
>;
type OnWriteFunction = Box<
    dyn Fn(SlaveId, ModbusAddress, ModbusDataType) -> std::result::Result<(), ExceptionCode>
        + Send
        + Sync,
>;

#[derive(Clone, PartialEq, Debug)]
pub struct ModbusSlaveConnectionParameters {
    pub allowed_slaves: Option<HashSet<SlaveId>>,
    pub allowed_ip_address: Option<HashSet<IpAddr>>,
    pub connection_time_to_live: Duration,
}

pub struct ModbusSlaveConnection {
    comm: ModbusSlaveCommunicationInfo,
    on_read: OnReadFunction,
    on_write: OnWriteFunction,
}

impl ModbusSlaveConnection {
    pub fn new_tcp(
        address: SocketAddr,
        on_read: OnReadFunction,
        on_write: OnWriteFunction,
    ) -> Self {
        let comm = ModbusSlaveCommunicationInfo::new_tcp(address);

        ModbusSlaveConnection {
            comm,
            on_read,
            on_write,
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

                let result = (self.on_write)(message_data.slave_id, address, params.value);

                if let Err(exception_code) = result {
                    return Ok(ModbusResponse::Error {
                        message_data,
                        exception_code,
                    });
                }

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
                    let result = (self.on_write)(message_data.slave_id, address.clone(), value);

                    if let Err(exception_code) = result {
                        return Ok(ModbusResponse::Error {
                            message_data,
                            exception_code,
                        });
                    }

                    results.push(result.unwrap());
                    address.address += 1;
                }

                let params = response::MultipleWriteResponse {
                    table: params.table,
                    address: params.starting_address,
                    ammount: results.len() as u16,
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

                    if let Err(exception_code) = result {
                        return Ok(ModbusResponse::Error {
                            message_data,
                            exception_code,
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
                    let result = (self.on_write)(
                        message_data.slave_id,
                        write_starting_address.clone(),
                        value,
                    );
                    if let Err(exception_code) = result {
                        return Ok(ModbusResponse::Error {
                            message_data,
                            exception_code,
                        });
                    }
                    write_starting_address.address += 1;
                }

                let mut read_starting_address = ModbusAddress {
                    table: params.table,
                    address: params.read_starting_address,
                };

                for _index in 0..params.read_ammount {
                    let result =
                        (self.on_read)(message_data.slave_id, read_starting_address.clone());

                    if let Err(exception_code) = result {
                        return Ok(ModbusResponse::Error {
                            message_data,
                            exception_code,
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

    pub async fn handle_connection(
        &self,
        mut socket: TcpStream,
        allowed_slaves: Option<HashSet<SlaveId>>,
        connection_time_to_live: Duration,
    ) -> Result<()> {
        loop {
            let bytes = match tokio::time::timeout(connection_time_to_live, socket.read()).await {
                Ok(Ok(bytes)) => bytes,
                Ok(Err(err)) => {
                    return Err(err);
                }
                Err(_) => {
                    break;
                }
            };

            if bytes.is_empty() {
                continue;
            }

            let queries =
                crate::messages::ModbusQuery::deserialize(bytes, ModbusSubprotocol::ModbusTCP)?;

            for query in queries {
                if allowed_slaves.as_ref().is_some()
                    && !allowed_slaves
                        .as_ref()
                        .unwrap()
                        .contains(&query.get_message_data().slave_id)
                {
                    continue;
                }

                let response = self.handle_query(query)?;
                ModbusSocket::write(
                    &mut socket,
                    response.serialize(ModbusSubprotocol::ModbusTCP)?,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn bind(&mut self) -> Result<()> {
        if !self.comm.is_bound() {
            self.comm.bind().await?;
        }
        Ok(())
    }
    pub async fn server_with_parameters(
        &mut self,
        params: ModbusSlaveConnectionParameters,
    ) -> Result<()> {
        self.bind().await?;

        let listener = self.comm.listener.as_ref().unwrap();
        loop {
            let (socket, addr) = listener.accept().await?;

            if params.allowed_ip_address.is_some()
                && !params
                    .allowed_ip_address
                    .as_ref()
                    .unwrap()
                    .contains(&addr.ip())
            {
                continue;
            }

            self.handle_connection(
                socket,
                params.allowed_slaves.clone(),
                params.connection_time_to_live,
            )
            .await?;
        }
    }

    pub fn serve(&mut self) -> impl std::future::Future<Output = Result<()>> + '_ {
        let params = ModbusSlaveConnectionParameters {
            allowed_ip_address: None,
            allowed_slaves: None,
            connection_time_to_live: Duration::from_secs(10),
        };

        self.server_with_parameters(params)
    }
}
