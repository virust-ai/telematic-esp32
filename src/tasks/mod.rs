mod can_rx;
mod mqtt;
mod quectel;
mod wifi;

use core::ffi::CStr;

pub use can_rx::can_receiver;
pub use mqtt::mqtt_handler;
pub use quectel::{quectel_rx_handler, quectel_tx_handler};
pub use wifi::{connection, net_task};

const MQTT_SERVERNAME: &str = "broker.bluleap.ai";
const SERVERNAME: &CStr = c"broker.bluleap.ai";
const MQTT_SERVERPORT: u16 = 8883;
const MQTT_CLIENT_ID: &str = "5680ff91-2d1c-4d0a-a8f7-f9c2a2066740";
const MQTT_USR_NAME: &str = "bike_test";
const MQTT_USR_PASS: [u8; 9] = *b"bike_test";
