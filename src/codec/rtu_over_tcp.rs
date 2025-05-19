use anyhow::Result;

pub trait ModbusRtuOverTcpSerialize
where
    Self: Sized,
{
    fn rtu_over_tcp_serialize(&self) -> Result<Vec<u8>>;
    fn rtu_over_tcp_deserialize(data: Vec<u8>) -> Result<Vec<Self>>;
}
