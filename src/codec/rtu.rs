use anyhow::Result;
pub trait ModbusRtuSerialize
where
    Self: Sized,
{
    fn rtu_serialize(&self) -> Result<Vec<u8>>;
    fn rtu_deserialize(data: Vec<u8>) -> Result<Vec<Self>>;
}
