mod codec;
mod connection;
mod messages;

#[cfg(test)]
mod test {
    use std::{net::SocketAddr, str::FromStr};

    use crate::{connection::ModbusConnection, messages::ModbusDataType};

    #[tokio::test]
    async fn test() {
        let inicio = std::time::Instant::now();
        
        let mut connection =
            ModbusConnection::new_tcp(SocketAddr::from_str("127.0.0.1:1026").unwrap());

        connection
            .add_read_holding_registers_query(0xFF, 1, 1)
            .unwrap();

        let result = connection.query().await.unwrap();

        assert_eq!(*result.get(&1).unwrap(), ModbusDataType::Register(0x66));

        let duracion = inicio.elapsed();
        println!("El test tardó: {:?}", duracion);
    }
}
