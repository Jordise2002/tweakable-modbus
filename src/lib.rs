mod codec;
mod connection;
mod messages;

pub use connection::ModbusConnection;
pub use connection::ModbusResult;

#[cfg(test)]
mod test {
    use super::*;
    use std::{net::SocketAddr, str::FromStr};

    use crate::{connection::ModbusConnection, messages::ModbusDataType};

    #[tokio::test]
    async fn test() {
        let inicio = std::time::Instant::now();

        let mut connection =
            ModbusConnection::new_tcp(SocketAddr::from_str("127.0.0.1:1026").unwrap());

        connection.add_write_coil_query(0xFF, 1, false).unwrap();
        connection.add_write_coil_query(0xFF, 0, false).unwrap();

        let result = connection.query().await.unwrap();

        println!("{:?}", result);

        for (address, data) in result {
            assert_eq!(
                data,
                ModbusResult::ReadResult(ModbusDataType::Register(address.address + 1))
            );
        }

        let duracion = inicio.elapsed();

        println!("El test tardó: {:?}", duracion);
    }
}
