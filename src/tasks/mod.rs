mod can_rx;
mod wifi;

pub use can_rx::can_receiver;
pub use wifi::{connection, net_task};
