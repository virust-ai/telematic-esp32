use embassy_net::{tcp::{State, TcpSocket}, Stack};
use embedded_io::{Read, Write};
use esp_println::println;
use esp_wifi::wifi::{
    WifiDevice, WifiStaDevice, WifiState
};
use mqttrust::{
    encoding::v4::{encode_slice, Connect, Protocol},
    MqttError,
    Packet,
    Publish,
    QoS,
};
use smoltcp::wire::IpAddress;

pub struct MqttClient<'a> {
    client_id: &'a str,
    stack: &'a Stack<WifiDevice<'a, WifiStaDevice>>,
    connection_state: bool,
    recv_buffer: [u8; 512],
    recv_index: usize,
    keep_alive_secs: Option<u16>,
    last_sent_millis: u64,
    current_millis_fn: fn() -> u64,
}

impl<'a> MqttClient<'a> {
    pub fn new(
        client_id: &'a str,
        stack: &'a Stack<WifiDevice<'a, WifiStaDevice>>,
        current_millis_fn: fn() -> u64,
    ) -> Self {
        MqttClient {
            client_id,
            stack,
            connection_state: false,
            recv_buffer: [0u8; 512],
            recv_index: 0,
            keep_alive_secs: None,
            last_sent_millis: 0,
            current_millis_fn,
        }
    }

    pub fn connect(
        &mut self,
        broker_addr: IpAddress,
        broker_port: u16,
        keep_alive_secs: u16,
        username: Option<&'a str>,
        password: Option<&'a [u8]>,
    ) -> Result<(), MqttError> {
        // self.keep_alive_secs = Some(keep_alive_secs);

        // if self.socket.state() != State::Closed {
        //     self.disconnect();
        // }

        // match esp_wifi::wifi::get_wifi_state() {
        //     WifiState::StaConnected => {},
        //     _ => {
        //         self.connection_state = false;
        //         return Err(MqttError::Overflow)
        //     },
        // }

        // self.socket.open(broker_addr, broker_port).unwrap();

        // let conn_pkt = Packet::Connect(Connect {
        //     protocol: Protocol::MQTT311,
        //     keep_alive: keep_alive_secs,
        //     client_id: self.client_id,
        //     clean_session: true,
        //     last_will: None,
        //     username,
        //     password,
        // });

        // self.send(conn_pkt)?;

        // match self.receive() {
        //     Ok(()) => {
        //         self.last_sent_millis = (self.current_millis_fn)();
        //         self.connection_state = true;
        //         Ok(())
        //     },
        //     Err(e) => Err(e),
        // }
        Ok(())
    }

    pub fn disconnect(&mut self) {
        // self.socket.disconnect();
        // self.connection_state = false;
    }

    pub fn publish(&mut self,
        topic_name: &str,
        payload: &[u8],
        qos: QoS
    ) -> Result<(), MqttError> {
        // let pub_pkt = Packet::Publish(Publish {
        //     dup: false,
        //     qos,
        //     pid: None,
        //     retain: false,
        //     topic_name,
        //     payload,
        // });

        // match esp_wifi::wifi::get_wifi_state() {
        //     WifiState::StaConnected => {},
        //     _ => {
        //         self.connection_state = false;
        //         return Err(MqttError::Overflow)
        //     },
        // }

        // self.send(pub_pkt)?;
        // self.last_sent_millis = (self.current_millis_fn)();
        Ok(())
    }

    pub fn poll(&mut self) {

        // if !self.socket.is_connected() {
        //     self.connection_state = false;
        //     return
        // }

        // if let Some(keep_alive_secs) = self.keep_alive_secs {
        //     if (self.last_sent_millis + (keep_alive_secs * 1000) as u64) < (self.current_millis_fn)() {
        //         let ping_err = self.send(Packet::Pingreq);
        //         match ping_err {
        //             Ok(()) => {
        //                 println!("Ping success");
        //                 self.last_sent_millis = (self.current_millis_fn)();
        //             },
        //             Err(_) => println!("Ping failed"),
        //         }
        //         return
        //     }
        // }
        // self.socket.work();
    }

    fn send(&mut self, packet: Packet<'_>) -> Result<(), MqttError> {
        // let mut buffer = [0u8; 1024];
        // let len = encode_slice(&packet, &mut buffer).unwrap();
        // match self.socket.write(&buffer[..len]) {
        //     Ok(_) => Ok(()),
        //     Err(_) => Err(MqttError::Overflow),
        // }
        Ok(())
    }

    fn receive(&mut self) -> Result<(), MqttError> {
        // loop {
        //     match self.socket.read(&mut self.recv_buffer) {
        //         Ok(len) => {
        //             self.recv_index = len;
        //             return Ok(());
        //         },
        //         Err(_) => return Err(MqttError::Overflow),
        //     }
        // }
        Ok(())
    }
}