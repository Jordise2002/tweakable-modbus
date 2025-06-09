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
            println!("adios {} {} {:?}", slave_id, address.address, value);
            return Ok(());
        });

        let mut slave = ModbusSlaveConnection::new_tcp(slave_address, on_read, on_write);

        slave.bind().await.unwrap();

        let allowed_slaves = vec![1];

        let allowed_ip_address = vec![IpAddr::from_str("127.0.0.1").unwrap()];

        slave
            .server_with_parameters(ModbusSlaveConnectionParameters::new(
                Some(allowed_slaves),
                Some(allowed_ip_address),
                Duration::from_secs(10),
            ))
            .await
            .unwrap();
    }
}
