use std::time::Duration;

use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream};

pub enum AddressingInfo {
    TcpConnection { address: SocketAddr },
    #[allow(dead_code)]
    RtuConnection { device: String, baud_rate: u32 },
}

//This trait is meant to abstract both TCP and RTU system sockets in order to unify behaviour
#[async_trait]
pub trait ModbusSocket {
    async fn read(&mut self) -> Result<Vec<u8>>;

    async fn write(&mut self, data: Vec<u8>) -> Result<()>;
}

#[async_trait]
impl ModbusSocket for TcpStream {
    async fn read(&mut self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut buffer = [0u8; 1024];

        loop {
            match tokio::time::timeout(
                Duration::from_millis(50),
                AsyncReadExt::read(self, &mut buffer),
            )
            .await
            {
                Ok(Ok(n)) => {
                    data.extend_from_slice(&buffer[..n]);
                    if n == 0 {
                        break;
                    }
                }
                Ok(Err(err)) => return Err(anyhow!(err.to_string())),
                Err(_) => {
                    break;
                }
            }
        }

        Ok(data)
    }

    async fn write(&mut self, data: Vec<u8>) -> Result<()> {
        match AsyncWriteExt::write(self, data.as_slice()).await {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err.to_string())),
        }
    }
}