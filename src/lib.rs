mod codec;
mod common;
mod communication;
mod master;
mod messages;
mod slave;

pub use master::ModbusMasterConnection;
pub use master::ModbusMasterConnectionParams;

pub use slave::ModbusSlaveConnection;
pub use slave::ModbusSlaveConnectionParameters;

pub use common::ModbusResult;

#[cfg(test)]
mod test {
    use std::{
        collections::HashSet,
        net::{IpAddr, SocketAddr},
        str::FromStr,
        time::Duration,
    };

    use crate::common::{ModbusAddress, ModbusDataType};

    use super::*;

    #[tokio::test]
    async fn test() {
        let slave_address = SocketAddr::from_str("127.0.0.1:1026").unwrap();

        let on_read = Box::new(|slave_id, address: ModbusAddress| {
            println!("hola {} {}", slave_id, address.address);
            return Ok(ModbusDataType::Register(22));
        });

        let on_write = Box::new(|slave_id, address: ModbusAddress, value: ModbusDataType| {
            print!("adios {} {} {:?}", slave_id, address.address, value);
            return Ok(());
        });



        let mut slave = ModbusSlaveConnection::new_tcp(slave_address, on_read, on_write);

        slave.bind().await.unwrap();

        let mut allowed_slaves = HashSet::new();

        allowed_slaves.insert(1);

        let mut allowed_ip_address = HashSet::new();

        allowed_ip_address.insert(IpAddr::from_str("127.0.0.1").unwrap());

        let handle = tokio::spawn(async move {
            slave
                .server_with_parameters(ModbusSlaveConnectionParameters {
                    allowed_slaves: Some(allowed_slaves),
                    allowed_ip_address: Some(allowed_ip_address),
                    connection_time_to_live: Duration::from_secs(10),
                })
                .await
                .unwrap();
        });

        handle.await;
    }
}
