mod rtu;
mod rtu_over_tcp;
mod tcp;

use rtu::ModbusRtuSerialize;
use rtu_over_tcp::ModbusRtuOverTcpSerialize;
use tcp::ModbusTcpSerialize;

use crate::connection::ModbusSubprotocol;

pub trait ModbusSerialize:
    ModbusRtuOverTcpSerialize + ModbusTcpSerialize + ModbusRtuSerialize
where
    Self: Sized,
{
    fn serialize(&self, subprotocol: ModbusSubprotocol) -> Result<Vec<u8>, String> {
        match subprotocol {
            ModbusSubprotocol::ModbusTCP => self.tcp_serialize(),
            ModbusSubprotocol::ModbusRTU => self.rtu_serialize(),
            ModbusSubprotocol::ModbusRTUOverTCP => self.rtu_over_tcp_serialize(),
        }
    }

    fn deserialize(data: Vec<u8>, subprotocol: ModbusSubprotocol) -> Result<Vec<Self>,String>
    {
        match subprotocol {
            ModbusSubprotocol::ModbusTCP => ModbusTcpSerialize::tcp_deserialize(data),
            ModbusSubprotocol::ModbusRTU => ModbusRtuSerialize::rtu_deserialize(data),
            ModbusSubprotocol::ModbusRTUOverTCP => ModbusRtuOverTcpSerialize::rtu_over_tcp_deserialize(data)
        }
    }
}
