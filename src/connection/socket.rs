use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
//This trait is meant to abstract both TCP and RTU system sockets in order to unify behaviour
#[async_trait]
pub trait ModbusSocket {
    async fn read(&mut self) -> Result<Vec<u8>, String>;

    async fn write(&mut self, data: Vec<u8>) -> Result<(), String>;
}

#[async_trait]
impl ModbusSocket for TcpStream {
    async fn read(&mut self) -> Result<Vec<u8>, String> {
        let mut data = Vec::new();
        let mut buffer = [0u8; 1024];

        loop {
            match AsyncReadExt::read(self, &mut buffer).await {
                Ok(n) => {
                    data.extend_from_slice(&buffer[..n]);
                    if n == 0 {
                        break;
                    }
                }
                Err(err) => return Err(err.to_string()),
            }
        }

        Ok(data)
    }

    async fn write(&mut self, data: Vec<u8>) -> Result<(), String> {
        match AsyncWriteExt::write(self, data.as_slice()).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }
}
