use core::{panic, str::FromStr};

use atat::{
    asynch::{AtatClient, Client},
    AtatIngress, DefaultDigester, Ingress, UrcChannel,
};
use core::fmt::Write;
use esp_hal::{
    gpio::Output,
    uart::{UartRx, UartTx},
    Async,
};
use esp_println::print;
use heapless::String;
use log::{error, info, warn};
use responses::FunctionalityLevelOfUE;

use crate::{
    at_command::{self, common::Urc},
    tasks::{MQTT_CLIENT_ID, MQTT_SERVERNAME, MQTT_SERVERPORT, MQTT_USR_NAME},
};
use at_command::common::general::*;

#[embassy_executor::task]
pub async fn quectel_tx_handler(
    mut client: Client<'static, UartTx<'static, Async>, 1024>,
    mut _pen: Output<'static>,
    mut _dtr: Output<'static>,
    urc_channel: &'static UrcChannel<at_command::common::Urc, 128, 3>,
) -> ! {
    let mut state: u32 = 0;
    let ca_chain = include_str!("../../crt.pem").as_bytes();
    let certificate = include_str!("../../dvt.crt").as_bytes();
    let private_key = include_str!("../../dvt.key").as_bytes();
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
            5 => {
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
            6 => {
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
            7 => {
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
            8 => {
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
            9 => {
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
            10 => {
                let mut raw_data = heapless::Vec::<u8, 4096>::new();
                raw_data.clear();
                info!("Quectel: list files");
                let mut subscriber = urc_channel.subscribe().unwrap();
                client.send(&FileList).await.unwrap();
                let now = embassy_time::Instant::now();
                loop {
                    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
                    match subscriber.try_next_message_pure() {
                        Some(Urc::ListFile(file)) => {
                            log::info!("File: {:?}", file);
                        }
                        Some(e) => {
                            error!("Unknown URC {:?}", e);
                        }
                        None => {
                            info!("Waiting for response...");
                        }
                    }
                    if now.elapsed().as_secs() > 10 {
                        break;
                    }
                }
                info!("Quectel: remove CA_CRT path");
                client
                    .send(&FileDel {
                        name: heapless::String::from_str("crt.pem").unwrap(),
                    })
                    .await;
                info!("Quectel: remove CLIENT_CRT path");
                client
                    .send(&FileDel {
                        name: heapless::String::from_str("dvt.crt").unwrap(),
                    })
                    .await;
                info!("Quectel: remove CLIENT_KEY path");
                client
                    .send(&FileDel {
                        name: heapless::String::from_str("dvt.key").unwrap(),
                    })
                    .await;
                // Upload CA cert
                info!("Quectel: Upload MQTT certs to quectel");
                raw_data.extend_from_slice(&ca_chain[0..1024]);
                client
                    .send(&FileUpl {
                        name: heapless::String::from_str("crt.pem").unwrap(),
                        size: 2574,
                    })
                    .await;
                client
                    .send(&SendRawData {
                        raw_data: raw_data.clone(),
                        len: 1024,
                    })
                    .await;
                raw_data.clear();
                raw_data.extend_from_slice(&ca_chain[1024..2048]);
                client
                    .send(&SendRawData {
                        raw_data: raw_data.clone(),
                        len: 1024,
                    })
                    .await;
                raw_data.clear();
                raw_data.extend_from_slice(&ca_chain[2048..]);
                client
                    .send(&SendRawData {
                        raw_data: raw_data.clone(),
                        len: 526,
                    })
                    .await;
                embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
                // Upload client cert
                client
                    .send(&FileUpl {
                        name: heapless::String::from_str("dvt.crt").unwrap(),
                        size: 1268,
                    })
                    .await;
                raw_data.clear();
                raw_data.extend_from_slice(&certificate[0..1024]);
                client
                    .send(&SendRawData {
                        raw_data: raw_data.clone(),
                        len: 1024,
                    })
                    .await;
                raw_data.clear();
                raw_data.extend_from_slice(&certificate[1024..]);
                client
                    .send(&SendRawData {
                        raw_data: raw_data.clone(),
                        len: 244,
                    })
                    .await;
                embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
                // Upload client key
                client
                    .send(&FileUpl {
                        name: heapless::String::from_str("dvt.key").unwrap(),
                        size: 1678,
                    })
                    .await;
                raw_data.clear();
                raw_data.extend_from_slice(&private_key[0..1024]);
                client
                    .send(&SendRawData {
                        raw_data: raw_data.clone(),
                        len: 1024,
                    })
                    .await;
                raw_data.clear();
                raw_data.extend_from_slice(&private_key[1024..]);
                client
                    .send(&SendRawData {
                        raw_data: raw_data.clone(),
                        len: 654,
                    })
                    .await;

                info!("Quectel: set MQTTS configuration");
                client
                    .send(&MqttConfig {
                        name: heapless::String::from_str("recv/mode").unwrap(),
                        param_1: Some(0),
                        param_2: Some(0),
                        param_3: Some(1),
                    })
                    .await;
                client
                    .send(&MqttConfig {
                        name: heapless::String::from_str("SSL").unwrap(),
                        param_1: Some(0),
                        param_2: Some(1),
                        param_3: Some(2),
                    })
                    .await;
                client
                    .send(&SslConfigCert {
                        name: heapless::String::from_str("cacert").unwrap(),
                        context_id: 2,
                        cert_path: Some(heapless::String::from_str("UFS:crt.pem").unwrap()),
                    })
                    .await;
                client
                    .send(&SslConfigCert {
                        name: heapless::String::from_str("clientcert").unwrap(),
                        context_id: 2,
                        cert_path: Some(heapless::String::from_str("UFS:dvt.crt").unwrap()),
                    })
                    .await;
                client
                    .send(&SslConfigCert {
                        name: heapless::String::from_str("clientkey").unwrap(),
                        context_id: 2,
                        cert_path: Some(heapless::String::from_str("UFS:dvt.key").unwrap()),
                    })
                    .await;
                client
                    .send(&SslConfigOther {
                        name: heapless::String::from_str("seclevel").unwrap(),
                        context_id: 2,
                        level: 2,
                    })
                    .await;
                client
                    .send(&SslConfigOther {
                        name: heapless::String::from_str("sslversion").unwrap(),
                        context_id: 2,
                        level: 4,
                    })
                    .await;
                client.send(&SslSetCipherSuite).await;
                client
                    .send(&SslConfigOther {
                        name: heapless::String::from_str("ignorelocaltime").unwrap(),
                        context_id: 2,
                        level: 1,
                    })
                    .await;
                client
                    .send(&MqttConfig {
                        name: heapless::String::from_str("version").unwrap(),
                        param_1: Some(0),
                        param_2: Some(4),
                        param_3: None,
                    })
                    .await;
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
                        server: heapless::String::from_str(MQTT_SERVERNAME).unwrap(),
                        port: MQTT_SERVERPORT,
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
                    let username = Some(String::<64>::from_str(MQTT_USR_NAME).unwrap());
                    let password = Some(String::<64>::from_str(MQTT_USR_NAME).unwrap());
                    let client_id: String<23> = match heapless::String::from_str("iot-tracker") {
                        Ok(id) => id,
                        Err(e) => {
                            error!("what the fuck ??? {:?}", e);
                            panic!()
                        }
                    };
                    match client
                        .send(&MqttConnect {
                            tcp_connect_id: 0,
                            client_id,
                            username,
                            password,
                        })
                        .await
                    {
                        Ok(_) => {
                            info!("Connecting to MQTT broker...");
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
                let mut mqtt_topic: heapless::String<128> = heapless::String::new();
                let mut payload: heapless::String<1024> = heapless::String::new();
                writeln!(
                    &mut mqtt_topic,
                    "channels/{}/messages/client/gps",
                    MQTT_CLIENT_ID
                )
                .unwrap();

                info!("Quectel: retrieve GPS RMC data");
                match client.send(&RetrieveGpsRmc).await {
                    Ok(res) => {
                        info!("\t {:?}", res);
                        writeln!(&mut payload, "{:?}", res).unwrap();
                        match client
                            .send(&MqttPublishExtended {
                                tcp_connect_id: 0,
                                msg_id: 0,
                                qos: 0,
                                retain: 0,
                                topic: mqtt_topic,
                                payload,
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
