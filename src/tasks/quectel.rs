use atat::{
    asynch::{AtatClient, Client},
    AtatIngress, DefaultDigester, Ingress,
};
use esp_hal::{
    uart::{UartRx, UartTx},
    Async,
};
use log::info;

use crate::quectel;

#[embassy_executor::task]
pub async fn quectel_tx_handler(mut client: Client<'static, UartTx<'static, Async>, 1024>) -> ! {
    let mut state: u8 = 0;
    info!("Quectel EG800k information:");
    loop {
        // These will all timeout after 1 sec, as there is no response
        match state {
            0 => {
                let res: quectel::common::general::responses::ManufacturerId = client
                    .send(&quectel::common::general::GetManufacturerId)
                    .await
                    .unwrap();
                info!("\t Manufacturer ID: {:?}", res);
            }
            1 => {
                let res: quectel::common::general::responses::ModelId = client
                    .send(&quectel::common::general::GetModelId)
                    .await
                    .unwrap();
                info!("\t Model ID: {:?}", res);
            }
            2 => {
                let res: quectel::common::general::responses::SoftwareVersion = client
                    .send(&quectel::common::general::GetSoftwareVersion)
                    .await
                    .unwrap();
                info!("\t Software Version: {:?}", res);
            }
            _ => {}
        }
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;

        state += 1;
    }
}

#[embassy_executor::task]
pub async fn quectel_rx_handler(
    mut ingress: Ingress<
        'static,
        DefaultDigester<quectel::common::Urc>,
        quectel::common::Urc,
        1024,
        128,
        3,
    >,
    mut reader: UartRx<'static, Async>,
) -> ! {
    ingress.read_from(&mut reader).await
}
