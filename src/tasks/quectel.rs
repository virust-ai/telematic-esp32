use core::str::FromStr;

use atat::{
    asynch::{AtatClient, Client},
    AtatIngress, DefaultDigester, Ingress, UrcChannel,
};
use esp_hal::{
    gpio::Output,
    uart::{UartRx, UartTx},
    Async,
};
use esp_println::print;
use log::{error, info, warn};
use responses::FunctionalityLevelOfUE;

use crate::at_command::{self, common::Urc};
use at_command::common::general::*;

#[embassy_executor::task]
pub async fn quectel_tx_handler(
    mut client: Client<'static, UartTx<'static, Async>, 1024>,
    mut _pen: Output<'static>,
    mut _dtr: Output<'static>,
    urc_channel: &'static UrcChannel<at_command::common::Urc, 128, 3>,
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
                // client
                //     .send(&SetFullFuncMode)
                //     .await
                //     .unwrap();
            }
            5 => {
                info!("Quectel: get Sim status");
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
                info!("Quectel: get GetNetworkInfo");
                match client.send(&GetNetworkInfo).await {
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
            10 => {
                info!("Quectel: set modem functionality to FULL");
                match client
                    .send(&SetUeFunctionality {
                        fun: FunctionalityLevelOfUE::Full,
                    })
                    .await
                {
                    Ok(_) => {
                        info!("OK");
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
            }
            11 => {
                let now = embassy_time::Instant::now();
                while now.elapsed().as_secs() < 30 {
                    match client.send(&GetEPSNetworkRegistrationStatus {}).await {
                        Ok(status) => {
                            log::info!("EPS network registration status: {:?}", status);
                            match status.stat {
                                1 => {
                                    let t = now.elapsed();
                                    info!("Registered (Home) after {} s", t.as_secs());
                                    break;
                                }
                                2 => {
                                    print!("."); // Searching
                                    continue;
                                }
                                3 => {
                                    error!("Registration denied");
                                    break;
                                }
                                4 => {
                                    error!("Registration failed");
                                    break;
                                }
                                5 => {
                                    let t = now.elapsed();
                                    info!("Registered (Roaming) after {} s", t.as_secs());
                                    break;
                                }
                                _ => {
                                    error!("Unknown registration status");
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("EPS network registration status not found: {:?}", e);
                        }
                    }
                }
            }
            12 => {
                info!("Quectel: connecting MQTT");
                match client
                    .send(&MqttOpen {
                        link_id: 0,
                        server: heapless::String::from_str("broker.hivemq.com").unwrap(),
                        port: 1883,
                    })
                    .await
                {
                    Ok(_) => {
                        info!("Connected to MQTT broker");
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
                let mut subscriber = urc_channel.subscribe().unwrap();
                loop {
                    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
                    match subscriber.try_next_message_pure() {
                        Some(Urc::MqttOpen(mqtopen_response)) => {
                            log::info!("MQTT Open response: {:?}", mqtopen_response);
                            match mqtopen_response.result {
                                0 => {
                                    info!("Connection opened");
                                    break;
                                }
                                _ => {
                                    error!("MQTT Open failed");
                                }
                            }
                        }
                        Some(e) => {
                            error!("Unknown URC {:?}", e);
                        }
                        None => {
                            info!("Waiting for response...");
                        }
                    }
                }
            }
            13 => {
                loop {
                    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
                    info!("Quectel: connecting MQTT");
                    match client
                        .send(&MqttConnect {
                            tcp_connect_id: 0,
                            client_id: heapless::String::from_str("bluleap").unwrap(),
                            username: None,
                            password: None,
                        })
                        .await
                    {
                        Ok(_) => {
                            info!("Connected to MQTT broker");
                            break;
                        }
                        Err(e) => {
                            error!("Failed to send AT command: {:?}", e);
                        }
                    }
                }

                let mut subscriber = urc_channel.subscribe().unwrap();
                loop {
                    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
                    match subscriber.try_next_message_pure() {
                        Some(Urc::MqttConnect(mqtconnect_response)) => {
                            log::info!("MQTT Open response: {:?}", mqtconnect_response);
                            match mqtconnect_response.result {
                                0 => {
                                    info!("Client connected");
                                    break;
                                }
                                _ => {
                                    error!("MQTT connect failed");
                                }
                            }
                        }
                        Some(e) => {
                            error!("Unknown URC {:?}", e);
                        }
                        None => {
                            info!("Waiting for response...");
                        }
                    }
                }
            }
            _ => {
                embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
                info!("Quectel: MQTT publish");
                match client
                    .send(&MqttPublishExtended {
                        tcp_connect_id: 0,
                        msg_id: 0,
                        qos: 0,
                        retain: 0,
                        topic: heapless::String::from_str("can/1").unwrap(),
                        payload: heapless::String::from_str("Hello from Quectel").unwrap(),
                    })
                    .await
                {
                    Ok(_) => {
                        info!("Published to MQTT broker");
                    }
                    Err(e) => {
                        error!("Failed to send AT command: {:?}", e);
                    }
                }
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
