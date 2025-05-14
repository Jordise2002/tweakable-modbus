use context::ModbusContext;
use socket::ModbusSocket;

mod context;
mod socket;

pub enum ModbusSubprotocol {
    ModbusTCP,
    ModbusRTU,
    ModbusRTUOverTCP
}

pub struct ModbusConnection
{
    comm: Box<dyn ModbusSocket>,
    context: ModbusContext,
    subprotocol: ModbusSubprotocol
}