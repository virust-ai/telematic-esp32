#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    prelude::*,
    timer::timg::TimerGroup,
    twai::{self, TwaiMode},
};

use log::{error, info};

#[derive(Debug)]
#[allow(dead_code)]
struct CanFrame {
    id: u32,
    data: [u8; 8],
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    info!("Welcome to esp-diag version: 0.1.2\r");

    let tx_pin = peripherals.GPIO1;
    let rx_pin = peripherals.GPIO10;
    const CAN_BAUDRATE: twai::BaudRate = twai::BaudRate::B250K;
    let mut twai_config = twai::TwaiConfiguration::new(
        peripherals.TWAI0,
        rx_pin,
        tx_pin,
        CAN_BAUDRATE,
        TwaiMode::Normal,
    ).into_async();
    twai_config.set_filter(
        const {
            twai::filter::SingleStandardFilter::new(
                b"xxxxxxxxxxx",
                b"x",
                [b"xxxxxxxx", b"xxxxxxxx"],
            )
        },
    );
    let mut can = twai_config.start();

    loop {
        match can.receive_async().await {
            Ok(frame) => {
                info!("Received CAN Frame: {:?}", frame);
            },
            Err(e) => {
                error!("Failed to receive CAN frame: {:?}", e);
            },
        }
    }
}