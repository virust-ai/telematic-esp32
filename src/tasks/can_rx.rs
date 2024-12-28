use esp_hal::twai::TwaiRx;
use log::{error, info};

use crate::{CanFrame, TwaiOutbox};
use embedded_can::{Frame, Id};

const MQTT_CAN_PACKET: u32 = 0x00001234;

#[embassy_executor::task]
pub async fn can_receiver(
    mut rx: TwaiRx<'static, esp_hal::Async>,
    channel: &'static TwaiOutbox,
) -> ! {
    info!("Hello CANRx !!\r");
    loop {
        let frame = rx.receive_async().await;

        match frame {
            Ok(frame) => {
                // repeat the frame back
                match frame.id() {
                    // ION doesn't work with StandardId
                    Id::Standard(_id) => {}
                    Id::Extended(id) => {
                        let mut data = [0u8; 8];
                        data[0..frame.data().len()].copy_from_slice(frame.data());

                        if id.as_raw() == MQTT_CAN_PACKET {
                            info!("Receive MQTT CAN packet");
                            // Try to send can frame to wifi task without blocking
                            let _ = channel.try_send(CanFrame {
                                id: id.as_raw(),
                                len: frame.dlc() as u8,
                                data,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                error!("TWAIT Received error: {:?}\r", e);
            }
        }
    }
}
