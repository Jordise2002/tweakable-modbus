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
use std::{collections::HashSet, time::Duration};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
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
    pub allowed_slaves: Arc<Option<HashSet<SlaveId>>>,
    pub allowed_ip_address: Option<HashSet<IpAddr>>,
    pub connection_time_to_live: Duration,
}

impl ModbusSlaveConnectionParameters {
    pub fn new(
        allowed_slaves: Option<Vec<SlaveId>>,
        allowed_ip_address: Option<Vec<IpAddr>>,
        connection_time_to_live: Duration,
    ) -> Self {
        let allowed_slaves = match allowed_slaves {
            Some(allowed_slaves) => Some(allowed_slaves.into_iter().collect::<HashSet<SlaveId>>()),
            None => None,
        };

        let allowed_slaves = Arc::new(allowed_slaves);

        let allowed_ip_address = match allowed_ip_address {
            Some(allowed_ip_address) => {
                Some(allowed_ip_address.into_iter().collect::<HashSet<IpAddr>>())
            }
            None => None,
        };

        Self {
            allowed_slaves,
            allowed_ip_address,
            connection_time_to_live,
        }
    }
}

pub struct ModbusSlaveConnectionContext {
    on_read: OnReadFunction,
    on_write: OnWriteFunction,
}
pub struct ModbusSlaveConnection {
    comm: ModbusSlaveCommunicationInfo,
    context: Arc<ModbusSlaveConnectionContext>,
}

impl ModbusSlaveConnection {
    pub fn new_tcp(
        address: SocketAddr,
        on_read: OnReadFunction,
        on_write: OnWriteFunction,
    ) -> Self {
        let comm = ModbusSlaveCommunicationInfo::new_tcp(address);

        let context = Arc::new(ModbusSlaveConnectionContext { on_read, on_write });

        ModbusSlaveConnection { comm, context }
    }

    pub fn handle_query(
        context: Arc<ModbusSlaveConnectionContext>,
        query: ModbusQuery,
    ) -> Result<ModbusResponse> {
        match query {
            ModbusQuery::SingleWriteQuery {
                message_data,
                params,
            } => {
                let address = ModbusAddress {
                    table: params.table,
                    address: params.starting_address,
                };

                let result = (context.on_write)(message_data.slave_id, address, params.value);

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
                    let result = (context.on_write)(message_data.slave_id, address.clone(), value);

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
                    let result = (context.on_read)(message_data.slave_id, address.clone());

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
                    let result = (context.on_write)(
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
                        (context.on_read)(message_data.slave_id, read_starting_address.clone());

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
        context: Arc<ModbusSlaveConnectionContext>,
        mut socket: TcpStream,
        allowed_slaves: Arc<Option<HashSet<SlaveId>>>,
        connection_time_to_live: Duration,
    ) -> Result<()> {
        print!("hola holita");
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
                if let Some(allowed_slaves) = allowed_slaves.as_ref() {
                    if !allowed_slaves.contains(&query.get_message_data().slave_id) {
                        continue;
                    }
                }

                let response = Self::handle_query(context.clone(), query)?;
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

            println!("Hola holita");

            if params.allowed_ip_address.is_some()
                && !params
                    .allowed_ip_address
                    .as_ref()
                    .unwrap()
                    .contains(&addr.ip())
            {
                continue;
            }

            let allowed_slaves = params.allowed_slaves.clone();
            let context = self.context.clone();
            let connection_time_to_live = params.connection_time_to_live.clone();

            tokio::spawn(async move {
                ModbusSlaveConnection::handle_connection(
                    context,
                    socket,
                    allowed_slaves,
                    connection_time_to_live,
                )
                .await
                .unwrap()
            });
        }
    }

    pub fn serve(&mut self) -> impl std::future::Future<Output = Result<()>> + '_ {
        let params = ModbusSlaveConnectionParameters::new(None, None, Duration::from_secs(10));

        self.server_with_parameters(params)
    }
}
