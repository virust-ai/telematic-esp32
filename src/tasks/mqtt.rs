use embassy_net::{
    tcp::{ConnectError, TcpSocket},
    Ipv4Address, Stack,
};
use embassy_time::{Duration, Instant, Timer};
use embedded_tls::{
    Aes128GcmSha256, Certificate, TlsConfig, TlsConnection, TlsContext, UnsecureProvider,
};
use esp_hal::rng::Trng;
use esp_println::println;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use log::{error, info};
// use rust_mqtt::{
//     client::{
//         client_config::{ClientConfig, MqttVersion},
//     },
//     packet::v5::reason_codes::ReasonCode,
//     utils::rng_generator::CountingRng,
// };

use crate::{dns::DnsBuilder, mqtt::MqttClient, TwaiOutbox};

#[embassy_executor::task]
pub async fn mqtt_handler(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    trng: &'static mut Trng<'static>,
    channel: &'static TwaiOutbox,
) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    //wait until wifi connected
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address); //dhcp IP address
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    loop {
        Timer::after(Duration::from_millis(1_000)).await;

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        let remote_endpoint = if let Ok(endpoint) = dns_query(stack).await {
            endpoint
        } else {
            continue
        };
        let mut mqtt_client = MqttClient::new("bluleap321", socket);
        mqtt_client.connect(remote_endpoint, 60, None, None).await.unwrap();
        loop {
            if let Ok(frame) = channel.try_receive() {
                use core::fmt::Write;
                let mut frame_str: heapless::String<80> = heapless::String::new();
                writeln!(&mut frame_str, "{:?}", frame).unwrap();
                if let Err(e) = mqtt_client.publish("can/1", frame_str.as_bytes(), mqttrust::QoS::AtMostOnce).await {
                    error!("Failed to publish MQTT packet: {:?}", e);
                    break;
                }
                info!("MQTT sent OK");
            }
            mqtt_client.poll().await;
            Timer::after_secs(1).await;
        }
        // println!("connecting...");
        // let connection = socket.connect(remote_endpoint).await;
        // if let Err(e) = connection {
        //     println!("connect error: {:?}", e);
        //     continue;
        // }
        // println!("connected!");
        // let mut read_record_buffer = [0; 1024];
        // let mut write_record_buffer = [0; 1024];
        // let ca = Certificate::X509(b"-----BEGIN CERTIFICATE-----\nMIIDyTCCArGgAwIBAgITCd7Xy+/NFK6oEDXLmP42JFvqMTANBgkqhkiG9w0BAQsF\nADAcMRowGAYDVQQDExFicm9rZXIuYmx1bGVhcC5haTAeFw0yNDEyMTgxNDAxNDVa\nFw0zNDEyMTYxNDAyMTVaMBwxGjAYBgNVBAMTEWJyb2tlci5ibHVsZWFwLmFpMIIB\nIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1VXaeza8lergGHVPXAUfjhPl\nVo6Yl4Tq++SymN1+rszvKNRo+kkRj7O41tl3MJi3jy/YHugjCoV/Ex6hrxejZ9h7\n8Qz4ufWWdbQMYa6Cwxso92rEzDZfPGs0RijvkeJGLN/KxIh0q36YYDGKDgJRNSTE\n7jYlDgrCvq5apPeOoHlivJdEH4fR6qX0gFPEFhQCtIXeyZQWmEEIOZzviLHqyUP7\nRPJ4DFMhF5jgk4PP6N/BcU8HM+89aELhriH8IsaFU+okW/TtpsTSXDLOlX1g1YH8\ni2IEptbvjpwr4JWQKftypSEIs0sm0TH2xEbzmJXd68R1VUyShc/uD/9Hl05UFQID\nAQABo4IBAjCB/zAOBgNVHQ8BAf8EBAMCAQYwDwYDVR0TAQH/BAUwAwEB/zAdBgNV\nHQ4EFgQUEIy1/Ugk8kWiG8xWGqMu8E72uJYwHwYDVR0jBBgwFoAUwCXI8yGdbiwB\nBvS1p5v8RSAZybwwQwYIKwYBBQUHAQEENzA1MDMGCCsGAQUFBzAChidodHRwczov\nL3ZhdWx0LmJsdWxlYXAuYWkvdjEvcGtpX21xdHQvY2EwHAYDVR0RBBUwE4IRYnJv\na2VyLmJsdWxlYXAuYWkwOQYDVR0fBDIwMDAuoCygKoYoaHR0cHM6Ly92YXVsdC5i\nbHVsZWFwLmFpL3YxL3BraV9tcXR0L2NybDANBgkqhkiG9w0BAQsFAAOCAQEAWmH4\nIE+megIv/9gxjOb9EYlahH2ooJYtB9IFIWeSDRoOUcMzD0Y3gGhUMkNqx6QgXBEJ\nkwqkaJYQoIj69e7W2FKC4kzp8vw5hh/BQTBfrz3y5qgBFjxZbBP3yFlwLKZzoJD+\n/3nBgRKqj2rc9V5RXDL/7KAfNoi1VQfJuGVx0rMUzA2B2kJ25cXyveGJOtFpnTQI\nEKd6xWn9or31GgqAH7Pqgo6PgxxvpgIfy1ji0tM65x/1bRBmkdDpt78TUwbxMhUI\nQU1RaBQFjCPfnN3qPMZ4kgrxfOQEEZJxOBdr35JPcJWLZ/vgw+suMn/TU9VDA7ib\neTlfnZ4DfEDD2/DkgQ==\n-----END CERTIFICATE-----\n-----BEGIN CERTIFICATE-----\nMIIDSDCCAjCgAwIBAgIUSHEal8sn2TIWZkubfMdLbPoFgcwwDQYJKoZIhvcNAQEL\nBQAwHDEaMBgGA1UEAxMRYnJva2VyLmJsdWxlYXAuYWkwHhcNMjQxMjE4MTM1OTEy\nWhcNMzQxMjE2MTM1OTQyWjAcMRowGAYDVQQDExFicm9rZXIuYmx1bGVhcC5haTCC\nASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAL1XVss//oobbX0cMk5MNwh8\nIyvHILdFFqkrY+KN6tUJbUgUJDgeBte1OPoCSQ2ivIVGNsHBq0PkV2SquNAwFBGl\nwYvwuyTf2+TUNpTafLL2uEfgQd/WG7+9wETD7el+kL6HoLnvxiQRupM5CwgIrLSj\nl/kICmeptYaaUISmdMWgz4ZcwMGVp15ItBgWF09CEEyvv6XU/ZmC5/0BiGRAy1Uk\nNH7ZCg5GoceXUHeIk0VUT4q2jU4PVzz326uvrSpsUiWNoqUaaLgPlPHU53+2l+di\n2bni4kJxKw2uR1HqmwAWT5AELka5nombgIlXuIGVfRwyTqf5FZjiGbuHL01NUisC\nAwEAAaOBgTB/MA4GA1UdDwEB/wQEAwIBBjAPBgNVHRMBAf8EBTADAQH/MB0GA1Ud\nDgQWBBTAJcjzIZ1uLAEG9LWnm/xFIBnJvDAfBgNVHSMEGDAWgBTAJcjzIZ1uLAEG\n9LWnm/xFIBnJvDAcBgNVHREEFTATghFicm9rZXIuYmx1bGVhcC5haTANBgkqhkiG\n9w0BAQsFAAOCAQEAQukZHqF15sFQDHpCwhHPbxGsndxueis8XOE2TAdyRWyMXB90\nVF/3vZrV25mdqCPfIW/CZklZppd2AXOilHFuEyWXSB1B4VpNhuTzIYUNyB0rf2Zu\nphZBAt4o4yZZHstGd3wU0eZrqyK+mTCOWKbDxiCALt8Glo0dZx3O0XMiQKAEU7wV\nyS1lQV351C6NtkL+KQVxeMLe8sjU3KiLpVulepW2IFofaT1bYUdLSe+3+iBEgM1Z\neQXxbga/ITXZAqle10O/N53EEHznZ3n7ZUv1qPgVLVdjo+xhiX8WqRCEFjagWJoR\n6qYJGDucUhTThoY0vJhAPyQ4IflwnAgSATHxiA==\n-----END CERTIFICATE-----");
        // let cert = Certificate::X509(b"-----BEGIN CERTIFICATE-----\nMIIDfTCCAmWgAwIBAgIUU6pp/eLjA7pUpCbCy9E3CNMuh2owDQYJKoZIhvcNAQEL\nBQAwHDEaMBgGA1UEAxMRYnJva2VyLmJsdWxlYXAuYWkwHhcNMjQxMjE4MTQwOTI1\nWhcNMjUwMzE4MTQwOTU1WjAvMS0wKwYDVQQDEyRkYzNkZmU4Ni04NjFhLTQ3YzUt\nOTk0Zi05ZWY5ZTE0NDI4ODIwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIB\nAQDgm4npQuVxRFFqRaKLptHLptOFhBfRJYZcaLygZUw2o6xPpe/WOQdoqJTgAYBR\nuh44ymkzgTSmIGWmkTmmgi/jYKEz4RRnF/v3gCcB1W/QyU/BqIaHtqH/SZMKSi46\n0H5MqWH/33wn6mk1xk3Sb0cTMWBbeCbZUWuQY3VFyJaxLQ0gY4ACYtN8nbkUORCh\n/LHYfxtZYvw083gD0RsfH/fUJmDw877Kp50Hxm3rQeBeA3rWYawDW42gWPHJcBB9\ni/8pXYq3l5vNUrcbUk69GeiyGLV84I2nghqLpRYLITXwcqCSoLK2xfdABoRSGYc+\nHlVG6wxGlPGwn/5oRejJoilHAgMBAAGjgaMwgaAwDgYDVR0PAQH/BAQDAgOoMB0G\nA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcDAjAdBgNVHQ4EFgQUYkJJEK3srplu\niLMzTcvAw+qUckEwHwYDVR0jBBgwFoAUEIy1/Ugk8kWiG8xWGqMu8E72uJYwLwYD\nVR0RBCgwJoIkZGMzZGZlODYtODYxYS00N2M1LTk5NGYtOWVmOWUxNDQyODgyMA0G\nCSqGSIb3DQEBCwUAA4IBAQAeDBO0uRpUAlQmCVXuxWxm62YjVNgkod5uJRpyAFa8\nmtH01cdUQQpEBLUrNoSbDpNXKbTWga3I3dE7GjwxLPSS1q6hUM+cWpwn9kFUSr/u\nHMIyofc03N/VBJVv7rUvflKtJepPwJ1oz532LLHnrddIeOtmqDgS/2C5mvprakle\nDcVqRtQ2Wq8SdRyS+320wxz23FfyboekZ8awqBaO8sAgXhdZePclCdpaO4rZTI4i\nS1K0UuSfck4wdUQOawAzQ+9V0GgXcrpYPR7D8kRuLxVNl5Dzb6QzqYdnDrVAtgYe\ntKWqI5U6MYbd0MZEWJuet3QQQMDwxyXSshgQv9BzXm1L\n-----END CERTIFICATE-----");
        // let tls_cfg = TlsConfig::new()
        //     .with_ca(ca)
        //     .with_cert(cert)
        //     .with_server_name("mqtts://broker.bluleap.ai")
        //     .enable_rsa_signatures();
        // let mut tls_connection =
        //     TlsConnection::new(socket, &mut read_record_buffer, &mut write_record_buffer);

        // let _ = tls_connection
        //     .open(TlsContext::new(
        //         &tls_cfg,
        //         UnsecureProvider::new::<Aes128GcmSha256>(&mut *trng),
        //     ))
        //     .await;

        // // mqttrust_core::Client::new("a", "123");
        // let mut config = ClientConfig::new(MqttVersion::MQTTv5, CountingRng(20000));
        // config.add_max_subscribe_qos(rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1);
        // config.add_username("bike_test");
        // config.add_password("bike_test");
        // config.add_client_id("dc3dfe86-861a-47c5-994f-9ef9e1442882");
        // config.max_packet_size = 100;
        // let mut recv_buffer = [0; 80];
        // let mut write_buffer = [0; 80];

        // let mut client = MqttClient::<_, 5, _>::new(
        //     tls_connection,
        //     &mut write_buffer,
        //     80,
        //     &mut recv_buffer,
        //     80,
        //     config,
        // );

        // match client.connect_to_broker().await {
        //     Ok(()) => {
        //         println!("MQTT boker connected")
        //     }
        //     Err(mqtt_error) => match mqtt_error {
        //         ReasonCode::NetworkError => {
        //             println!("MQTT Network Error");
        //             continue;
        //         }
        //         _ => {
        //             println!("Other MQTT Error: {:?}", mqtt_error);
        //             continue;
        //         }
        //     },
        // }
        // loop {
        //     let frame = channel.receive().await;
        //     use core::fmt::Write;
        //     let mut frame_str: heapless::String<80> = heapless::String::new();
        //     writeln!(&mut frame_str, "{:?}", frame).unwrap();
        //     match client
        //         .send_message(
        //             "channels/dc3dfe86-861a-47c5-994f-9ef9e1442882/messages/can",
        //             frame_str.as_bytes(),
        //             rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1,
        //             true,
        //         )
        //         .await
        //     {
        //         Ok(()) => {
        //             println!("sent CAN packet");
        //         }
        //         Err(mqtt_error) => match mqtt_error {
        //             ReasonCode::NetworkError => {
        //                 println!("MQTT Network Error");
        //                 continue;
        //             }
        //             _ => {
        //                 println!("Other MQTT Error: {:?}", mqtt_error);
        //                 continue;
        //             }
        //         },
        //     }
        //     Timer::after(Duration::from_millis(10)).await;
        // }
    }
}

pub async fn dns_query(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
) -> Result<embassy_net::IpEndpoint, ConnectError> {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
    let mut buffer = [0; 512];
    let dns_ip = Ipv4Address::new(8, 8, 8, 8);
    let remote_endpoint = embassy_net::IpEndpoint {
        addr: embassy_net::IpAddress::Ipv4(dns_ip),
        port: 53,
    };
    socket.connect(remote_endpoint).await?;
    let dns_builder = DnsBuilder::build("broker.hivemq.com");
    socket.write(&dns_builder.query_data()).await.unwrap();

    let size = socket.read(&mut buffer).await.unwrap();
    let broker_ip = if size > 2 {
        if let Ok(ips) = DnsBuilder::parse_dns_response(&buffer[2..size]) {
            info!("broker IP: {}.{}.{}.{}", ips[0], ips[1], ips[2], ips[3]);
            ips
        } else {
            return Err(ConnectError::NoRoute);
        }
    } else {
        return Err(ConnectError::NoRoute);
    };

    let broker_ipv4 = Ipv4Address::new(broker_ip[0], broker_ip[1], broker_ip[2], broker_ip[3]);

    let remote_endpoint = embassy_net::IpEndpoint {
        addr: embassy_net::IpAddress::Ipv4(broker_ipv4),
        port: 1883,
    };
    Ok(remote_endpoint)
}
