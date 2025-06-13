use embassy_net::Runner;
use embassy_time::{Duration, Timer};
use esp_println::println;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
    WifiState,
};

use crate::cfg::net_cfg::{WIFI_PSWD, WIFI_SSID};

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    println!("INFO - Start the connection task");
    println!(
        "INFO - Device capabilities: {:?}",
        controller.capabilities()
    );
    println!("INFO - Turn off power saving mode");
    controller
        .set_power_saving(esp_wifi::config::PowerSaveMode::None)
        .unwrap();
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: WIFI_SSID.try_into().unwrap(),
                password: WIFI_PSWD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("INFO - Start WIFI");
            controller.start_async().await.unwrap();
        }
        println!("INFO - Connecting the WIFI...");

        match controller.connect_async().await {
            Ok(_) => println!("INFO - Wifi connected!"),
            Err(e) => {
                println!("ERROR - Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await
}
