pub trait ModbusRtuOverTcpSerialize
where
    Self: Sized,
{
    fn rtu_over_tcp_serialize(&self) -> Result<Vec<u8>, String>;
    fn rtu_over_tcp_deserialize(data: Vec<u8>) -> Result<Self, String>;
}
