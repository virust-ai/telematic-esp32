#![no_std]
#![feature(type_alias_impl_trait)]

// Import all the core macros needed by Serde in no_std environment
use crate::util::no_std_prelude::*;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    clock::ClockControl,
    embassy,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
    prelude::*,
    spi::{master::Spi, SpiMode},
    system::SystemExt,
    timer::TimerGroup,
};
use esp_println::println;
use static_cell::StaticCell;

// Create a module structure that includes util
mod util {
    pub use crate::telematic_esp32::util::no_std_prelude;
}

// Import the telematic_esp32 crate
extern crate telematic_esp32;

// Include the flash driver module directly
#[path = "../../../app/src/hal/flash/mod.rs"]
mod hal_flash;

// Create a module structure that matches the crate
mod hal {
    pub mod flash {
        pub use crate::hal_flash::*;
    }
}

use hal::flash::W25Q128FVSG;

// Define static cell to hold peripherals
static PERIPHERALS: StaticCell<Peripherals> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize ESP HAL for ESP32C6
    println!("Initializing HAL...");
    let peripherals = PERIPHERALS.init(Peripherals::take());
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0);
    println!("HAL Initialized.");

    // Run the test directly
    test_flash_communication().await;
    println!("Test completed.");
}

async fn test_flash_communication() {
    println!("Starting flash communication test...");

    // Initialize SPI for flash communication
    let peripherals = unsafe { PERIPHERALS.get_ref() };
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Configure SPI Pins for ESP32-C6
    // IMPORTANT: Verify these GPIO pins match your ESP32-C6 board's wiring for the flash chip.
    let sclk = io.pins.gpio6;
    let mosi = io.pins.gpio7;
    let miso = io.pins.gpio8;
    let cs_pin = io.pins.gpio9;

    println!("Initializing SPI...");
    let mut config = esp_hal::spi::master::Config::default();
    config.baudrate = 100u32.kHz();
    config.data_mode = SpiMode::Mode0;

    let spi = Spi::new_blocking(peripherals.SPI2, sclk, mosi, miso, config);

    // Software CS pin control
    let cs = Output::new(cs_pin, Level::High);

    println!("Initializing Flash Driver...");
    // Use the actual driver imported from the crate
    let mut flash = W25Q128FVSG::new(spi, cs);

    // Initialize the flash chip
    flash.init().await;
    println!("Flash initialized.");

    println!("Reading JEDEC ID...");
    let id = flash.read_id().await;
    println!("JEDEC ID: {:02x?}", id);

    // Test writing and reading data
    let address = 0x1000;
    let mut write_data = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut read_data = [0u8; 4];

    println!(
        "Writing data to address {:#08x}: {:02x?}",
        address, write_data
    );
    flash.write_data(address, &mut write_data).await;

    // Add delay for write completion
    Timer::after(Duration::from_millis(10)).await;

    println!("Reading data from address {:#08x}...", address);
    flash.read_data(address, &mut read_data).await;

    println!("Read Data: {:02x?}", read_data);
    assert!(read_data == write_data, "Write/Read data mismatch");
    println!("Write/Read test passed.");

    // Test sector erase
    let sector_num = address / 4096; // Convert address to sector number
    println!(
        "Erasing sector {} at address {:#08x}...",
        sector_num,
        sector_num * 4096
    );
    flash.erase_sector(sector_num).await;

    // Add delay for erase completion
    Timer::after(Duration::from_millis(100)).await;

    // Verify erase by reading back
    println!("Verifying erase at address {:#08x}...", address);
    flash.read_data(address, &mut read_data).await;

    println!("Read after erase: {:02x?}", read_data);
    assert!(
        read_data == [0xFF, 0xFF, 0xFF, 0xFF],
        "Sector erase verification failed"
    );
    println!("Sector erase test passed.");

    println!("Flash test finished successfully.");
}
