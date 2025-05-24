use std::{net::{SocketAddr}};
use tokio::net::TcpStream;
use crate::connection::socket::ModbusSocket;

use anyhow::{anyhow, Result};
pub enum AddressingInfo {
    TcpConnection { address: SocketAddr },
    RtuConnection { device: String, baud_rate: u32 },
}

pub struct ModbusCommunicationInfo {
    pub comm: Option<Box<dyn ModbusSocket>>,
    addressing_info: AddressingInfo,
}

impl ModbusCommunicationInfo {
    pub fn new_tcp(address: SocketAddr) -> Self {
        ModbusCommunicationInfo {
            comm: None,
            addressing_info: AddressingInfo::TcpConnection { address },
        }
    }

    pub fn new_rtu(device: String, baud_rate: u32) -> Self {
        ModbusCommunicationInfo {
            comm: None,
            addressing_info: AddressingInfo::RtuConnection { device, baud_rate },
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
 
        if let AddressingInfo::TcpConnection { address } = &self.addressing_info {
            let stream = TcpStream::connect(address).await;
            if let Ok(stream) = stream
            {
                self.comm = Some(Box::new(stream));
                return Ok(());
            }
        }
        
        return Err(anyhow!("Couldn't open connection"));
    }
}
