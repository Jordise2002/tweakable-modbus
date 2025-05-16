pub trait ModbusTcpSerialize
where
    Self: Sized,
{
    fn tcp_serialize(&self) -> Result<Vec<u8>, String>;
    fn tcp_desrialize(data: Vec<u8>) -> Result<Self, String>;
}
