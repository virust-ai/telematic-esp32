use atat::{
    asynch::{AtatClient, Client},
    AtatIngress, DefaultDigester, Ingress,
};
use esp_hal::{
    gpio::Output,
    uart::{UartRx, UartTx},
    Async,
};
use log::{error, info};

use crate::quectel;

#[embassy_executor::task]
pub async fn quectel_tx_handler(
    mut client: Client<'static, UartTx<'static, Async>, 1024>,
    mut _pen: Output<'static>,
    mut _dtr: Output<'static>,
) -> ! {
    let mut state: u32 = 0;
    loop {
        // These will all timeout after 1 sec, as there is no response
        match state {
            0 => {
                client
                    .send(&quectel::common::general::DisableEchoMode)
                    .await
                    .unwrap();
            }
            1 => {
                let res: quectel::common::general::responses::ManufacturerId = client
                    .send(&quectel::common::general::GetManufacturerId)
                    .await
                    .unwrap();
                info!("\t {:?}", res);
            }
            2 => {
                let res: quectel::common::general::responses::ModelId = client
                    .send(&quectel::common::general::GetModelId)
                    .await
                    .unwrap();
                info!("\t {:?}", res);
            }
            3 => {
                let res: quectel::common::general::responses::SoftwareVersion = client
                    .send(&quectel::common::general::GetSoftwareVersion)
                    .await
                    .unwrap();
                info!("\t {:?}", res);
            }
            4 => {
                // let res: quectel::common::general::responses::CommonResponse = client
                //     .send(&quectel::common::general::SetFullFuncMode)
                //     .await
                //     .unwrap();
                // info!("\t {:?}", res);
            }
            5 => {
                let res: quectel::common::general::responses::SimCardStatus = client
                    .send(&quectel::common::general::GetSimCardStatus)
                    .await
                    .unwrap();
                info!("\t {:?}", res);
            }
            6 => {
                let res: quectel::common::general::responses::NetworkSignalQuality = client
                    .send(&quectel::common::general::GetNetworkSignalQuality)
                    .await
                    .unwrap();
                info!("\t {:?}", res);
            }
            7 => {
                let res: quectel::common::general::responses::NetworkOperatorName = client
                    .send(&quectel::common::general::GetNetworkOperatorName)
                    .await
                    .unwrap();
                info!("\t {:?}", res);
            }
            8 => {
                client
                    .send(&quectel::common::general::EnableGpsFunc)
                    .await
                    .unwrap();
            }
            9 => {
                client
                    .send(&quectel::common::general::EnableAssistGpsFunc)
                    .await
                    .unwrap();
            }
            _ => {
                match client
                    .send(&quectel::common::general::RetrieveGpsData)
                    .await
                {
                    Ok(res) => {
                        info!("\t {:?}", res);
                    }
                    Err(e) => {
                        error!("\t {:?}", e);
                    }
                }
            }
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
