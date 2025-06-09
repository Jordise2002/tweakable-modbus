mod codec;
mod common;
mod communication;
mod master;
mod messages;
mod slave;

pub use master::ModbusMasterConnection;
pub use master::ModbusMasterConnectionParams;

pub use slave::ModbusSlaveConnection;
pub use slave::ModbusSlaveConnectionParameters;

pub use common::ModbusResult;