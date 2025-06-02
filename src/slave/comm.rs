use anyhow::{anyhow, Result};
use std::net::{SocketAddr};
use tokio::net::TcpListener;
use crate::communication::AddressingInfo;

pub struct ModbusSlaveCommunicationInfo {
    pub listener: Option<TcpListener>,
    addressing_info: AddressingInfo
}

impl ModbusSlaveCommunicationInfo {
    pub fn new_tcp(address: SocketAddr) -> Self
    {
        let addressing_info = AddressingInfo::TcpConnection { address };

        ModbusSlaveCommunicationInfo { listener: None, addressing_info}
    }

    pub async fn bind(& mut self) -> Result<()>
    {
       if let AddressingInfo::TcpConnection { address } = self.addressing_info {
            self.listener = Some(TcpListener::bind(address).await?);

       }
       
       return Err(anyhow!("Rtu is not supported"));
    }

    pub fn is_bound(& self) -> bool
    {
        return self.listener.is_some()
    }
}