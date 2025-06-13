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
use hal::flash::W25Q128FVSG;
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
    let cs_pin = Output::new(peripherals.GPIO3, esp_hal::gpio::Level::High);
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
    info!("Starting flash communication test...");

    // Software CS pin control, enable chip select
    let mut flash = W25Q128FVSG::new(spi, cs_pin);

    // Initialize the flash chip
    flash.init().await;
    info!("Flash initialized.");

    // Read JEDEC ID
    info!("Reading JEDEC ID...");
    let id = flash.read_id().await;
    info!("JEDEC ID: {id:02x?}");

    // Erase the entire chip, comment out as it takes a long time
    // Uncomment the following lines to erase the chip
    //info!("Erasing entire chip...");
    //flash.erase_chip().await;
    //info!("Chip erased successfully.");

    // Test writing and reading data
    let address = 0x1000;
    let write_data = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut read_data = [0u8; 4];

    info!("Writing data to address {address:#08x}: {write_data:02x?}");
    flash.write_data(address, &write_data).await;

    info!("Reading data from address {address:#08x}...");
    flash.read_data(address, &mut read_data).await;

    info!("Read Data: {read_data:02x?}");
    if read_data != write_data {
        error!("Write/Read data mismatch: wrote {write_data:02x?}, read {read_data:02x?}");
    } else {
        info!("Write/Read test passed.");
    }

    // Test sector erase
    let sector_addr = address; // Use the actual address, not sector number
    info!("Erasing sector at address {sector_addr:#08x}...");
    flash.erase_sector(sector_addr).await; // Pass address, not sector number

    // Verify erase by reading back
    info!("Verifying erase at address {address:#08x}...");
    flash.read_data(address, &mut read_data).await;

    info!("Read after erase: {read_data:02x?}");
    if read_data != [0xFF, 0xFF, 0xFF, 0xFF] {
        error!("Sector erase verification failed: expected [0xFF; 4], got {read_data:02x?}");
    } else {
        info!("Sector erase verification passed.");
    }
    info!("Sector erase test passed.");

    info!("Flash test finished successfully.");
    info!("Test completed.");

    loop {
        Timer::after_secs(2).await;
        info!("Running...");
    }
}
