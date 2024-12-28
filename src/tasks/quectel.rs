use atat::{
    asynch::{AtatClient, Client},
    AtatIngress, DefaultDigester, Ingress,
};
use esp_hal::{
    gpio::Output,
    uart::{UartRx, UartTx},
    Async,
};
use log::{error, info, warn};

use crate::at_command;
use at_command::common::general::*;

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
                info!("Quectel: disable echo mode");
                if let Err(e) = client.send(&DisableEchoMode).await {
                    error!("Failed to send AT command: {:?}", e);
                }
            }
            1 => {
                info!("Quectel: get ManufacturerId");
                match client.send(&GetManufacturerId).await {
                    Ok(res) => {
                        info!("\t {:?}", res);
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            2 => {
                info!("Quectel: get ModelId");
                match client.send(&GetModelId).await {
                    Ok(res) => {
                        info!("\t {:?}", res);
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            3 => {
                info!("Quectel: get SoftwareVersion");
                match client.send(&GetSoftwareVersion).await {
                    Ok(res) => {
                        info!("\t {:?}", res);
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            4 => {
                // let res: quectel::common::general::responses::CommonResponse = client
                //     .send(&quectel::common::general::SetFullFuncMode)
                //     .await
                //     .unwrap();
                // info!("\t {:?}", res);
            }
            5 => {
                info!("Quectel: get SoftwareVersion");
                match client.send(&GetSimCardStatus).await {
                    Ok(res) => {
                        info!("\t {:?}", res);
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            6 => {
                info!("Quectel: get NetworkSignalQuality");
                match client.send(&GetNetworkSignalQuality).await {
                    Ok(res) => {
                        info!("\t {:?}", res);
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            7 => {
                info!("Quectel: get NetworkOperatorName");
                match client.send(&GetNetworkOperatorName).await {
                    Ok(res) => {
                        info!("\t {:?}", res);
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            8 => {
                info!("Quectel: enable GPS functionality");
                match client.send(&EnableGpsFunc).await {
                    Ok(_) => {
                        info!("Quectel: enable GPS functionality OK");
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            9 => {
                info!("Quectel: enable Assist GPS functionality");
                match client.send(&EnableAssistGpsFunc).await {
                    Ok(_) => {
                        info!("Quectel: enable Assist GPS functionality OK");
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            _ => {
                info!("Quectel: retrieve GPS RMC data");
                match client.send(&RetrieveGpsRmc).await {
                    Ok(res) => {
                        info!("\t {:?}", res);
                    }
                    Err(e) => {
                        warn!("\t Failed to get GPS data: {:?}", e);
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
        DefaultDigester<at_command::common::Urc>,
        at_command::common::Urc,
        1024,
        128,
        3,
    >,
    mut reader: UartRx<'static, Async>,
) -> ! {
    ingress.read_from(&mut reader).await
}
