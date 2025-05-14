use context::ModbusContext;
use socket::ModbusSocket;

mod context;
mod socket;

pub struct ModbusConnection
{
    comm: Box<dyn ModbusSocket>,
    context: ModbusContext
}