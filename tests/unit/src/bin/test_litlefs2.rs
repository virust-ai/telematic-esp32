#![no_std]
#![no_main]
#[allow(unused_imports)]
// Include the flash driver module directly
#[path = "../../../../app/src/hal/flash/mod.rs"]
mod hal_flash;

mod hal {
    pub mod flash {
        pub use crate::hal_flash::*;
    }
}
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::time::RateExtU32;
use esp_hal::{
    clock::CpuClock,
    gpio::Output,
    spi::{
        master::{Config, Spi},
        Mode,
    },
    timer::timg::TimerGroup,
};
#[allow(unused_imports)]
use hal::flash::W25Q128FVSG;
#[allow(unused_imports)]
use log::{error, info};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize ESP HAL for ESP32C6
    info!("Initializing HAL...");
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    // Initialize the timer group for embassy
    esp_alloc::heap_allocator!(200 * 1024);
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);
    // Initialize peripherals
    let sclk = peripherals.GPIO18;
    let miso = peripherals.GPIO20;
    let mosi = peripherals.GPIO19;
    #[allow(unused_variables)]
    let cs_pin = Output::new(peripherals.GPIO3, esp_hal::gpio::Level::High);
    #[allow(unused_variables)]
    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(100_u32.kHz())
            .with_mode(Mode::_0), // CPOL = 0, CPHA = 0 (Mode 0 per datasheet)
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_miso(miso);

    // Run the test directly
    info!("Starting little file system test...");

    // Wait for the test ending
    loop {
        Timer::after_secs(2).await;
        info!("Running...");
    }
}
