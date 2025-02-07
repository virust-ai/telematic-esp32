mod can_rx;
mod mqtt;
mod quectel;
mod wifi;

pub use can_rx::can_receiver;
pub use mqtt::mqtt_handler;
pub use quectel::{quectel_rx_handler, quectel_tx_handler};
pub use wifi::{connection, net_task};

const MQTT_SERVERNAME: &str = "broker-s.ionmobility.net";
const MQTT_CLIENT_ID: &str = "7f9ea02a-c93a-4b9d-b638-bf989865f9fe";
const MQTT_USR_NAME: &str = "37c63374-e927-4a32-a2a2-ad8bfb7ee945";
const MQTT_USR_PASS: [u8; 36] = *b"4b8ce895-83af-4f61-b469-f97ee2f18d4b";
