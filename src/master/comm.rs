use crate::communication::{AddressingInfo, ModbusSocket};
use std::net::SocketAddr;
use tokio::net::TcpStream;

use anyhow::{anyhow, Result};

pub struct ModbusMasterCommunicationInfo {
    pub comm: Option<Box<dyn ModbusSocket>>,
    addressing_info: AddressingInfo,
}

impl ModbusMasterCommunicationInfo {
    pub fn new_tcp(address: SocketAddr) -> Self {
        ModbusMasterCommunicationInfo {
            comm: None,
            addressing_info: AddressingInfo::TcpConnection { address },
        }
    }

    #[allow(dead_code)]
    pub fn new_rtu(device: String, baud_rate: u32) -> Self {
        ModbusMasterCommunicationInfo {
            comm: None,
            addressing_info: AddressingInfo::RtuConnection { device, baud_rate },
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        if let AddressingInfo::TcpConnection { address } = &self.addressing_info {
            let stream = TcpStream::connect(address).await;
            if let Ok(stream) = stream {
                self.comm = Some(Box::new(stream));
                return Ok(());
            }
        }

        return Err(anyhow!("Couldn't open connection"));
    }

    pub async fn is_connected(&mut self) -> bool {
        if self.comm.is_none() {
            return false;
        }

        self.comm.as_mut().unwrap().is_open().await
    }
}
