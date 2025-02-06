mod can_rx;
mod mqtt;
mod quectel;
mod wifi;

pub use can_rx::can_receiver;
pub use mqtt::mqtt_handler;
pub use quectel::{quectel_rx_handler, quectel_tx_handler};
pub use wifi::{connection, net_task};

const MQTT_CLIENT_ID: &str = "5680ff91-2d1c-4d0a-a8f7-f9c2a2066740";
