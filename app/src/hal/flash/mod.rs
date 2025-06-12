// Import core macros needed by Serde in no_std environment
#[allow(unused_imports)]
use core::concat;
#[allow(unused_imports)]
use core::debug_assert_eq;
#[allow(unused_imports)]
use core::format_args;
#[allow(unused_imports)]
use core::marker::Sized;
#[allow(unused_imports)]
use core::option::Option;
#[allow(unused_imports)]
use core::option::Option::{None, Some};
#[allow(unused_imports)]
use core::panic;
#[allow(unused_imports)]
use core::result::Result;
#[allow(unused_imports)]
use core::result::Result::{Err, Ok};
#[allow(unused_imports)]
use core::stringify;
#[allow(unused_imports)]
use core::unimplemented;
#[allow(unused_imports)]
use core::write;

use core::marker::PhantomData;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Output;
#[allow(unused_imports)]
use esp_hal::spi::master::Config;
use esp_hal::spi::master::Spi;
use esp_hal::Blocking;

#[allow(dead_code)]
// SPI Commands from W25Q128FV datasheet
const WRITE_ENABLE: u8 = 0x06;
const WRITE_DISABLE: u8 = 0x04;
const READ_STATUS_REG1: u8 = 0x05;
const READ_STATUS_REG2: u8 = 0x35;
const READ_STATUS_REG3: u8 = 0x15;
const WRITE_STATUS_REG1: u8 = 0x01;
const WRITE_STATUS_REG2: u8 = 0x31;
const WRITE_STATUS_REG3: u8 = 0x11;
const READ_JEDEC_ID: u8 = 0x9F;
const READ_DATA: u8 = 0x03;
const FAST_READ: u8 = 0x0B;
const PAGE_PROGRAM: u8 = 0x02;
const SECTOR_ERASE_4KB: u8 = 0x20;
const BLOCK_ERASE_32KB: u8 = 0x52;
const BLOCK_ERASE_64KB: u8 = 0xD8;
const CHIP_ERASE: u8 = 0xC7;
const POWER_DOWN: u8 = 0xB9;
const RELEASE_POWER_DOWN: u8 = 0xAB;
const ENABLE_RESET: u8 = 0x66;
const RESET_DEVICE: u8 = 0x99;

// Status Register 1 bits
const BUSY_BIT: u8 = 0x01;
const WEL_BIT: u8 = 0x02;

// Timing constants (from datasheet)
const PAGE_SIZE: usize = 256;
const SECTOR_SIZE: usize = 4096;
const BLOCK_32K_SIZE: usize = 32768;
const BLOCK_64K_SIZE: usize = 65536;

pub struct W25Q128FVSG<'d> {
    spi: Spi<'d, Blocking>,
    cs: Output<'d>,
    _mode: PhantomData<Blocking>,
}

impl<'d> W25Q128FVSG<'d> {
    pub fn new(spi: Spi<'d, Blocking>, cs: Output<'d>) -> Self {
        Self {
            spi,
            cs,
            _mode: PhantomData,
        }
    }

    pub async fn init(&mut self) {
        // Initialize CS pin high
        self.cs.set_high();
        Timer::after(Duration::from_millis(10)).await;

        // Release from power-down if needed
        self.release_power_down().await;
        Timer::after(Duration::from_millis(1)).await;
    }

    /// Read JEDEC ID (Manufacturer ID + Device ID)
    pub async fn read_id(&mut self) -> [u8; 3] {
        let mut id = [0u8; 3];

        self.cs.set_low();
        // Send command
        self.spi.write_bytes(&[READ_JEDEC_ID]).unwrap();
        // Read 3 bytes of ID
        self.spi.transfer(&mut id).unwrap();
        self.cs.set_high();

        id
    }

    /// Read status register 1
    pub async fn read_status_reg1(&mut self) -> u8 {
        let mut status = [0u8; 1];

        self.cs.set_low();
        self.spi.write_bytes(&[READ_STATUS_REG1]).unwrap();
        self.spi.transfer(&mut status).unwrap();
        self.cs.set_high();

        status[0]
    }

    /// Check if device is busy (programming/erasing)
    pub async fn is_busy(&mut self) -> bool {
        let status = self.read_status_reg1().await;
        (status & BUSY_BIT) != 0
    }

    /// Wait for device to become ready
    pub async fn wait_ready(&mut self) {
        while self.is_busy().await {
            Timer::after(Duration::from_millis(1)).await;
        }
    }

    /// Check if write enable latch is set
    pub async fn is_write_enabled(&mut self) -> bool {
        let status = self.read_status_reg1().await;
        (status & WEL_BIT) != 0
    }

    /// Send write enable command
    pub async fn write_enable(&mut self) {
        self.cs.set_low();
        self.spi.write_bytes(&[WRITE_ENABLE]).unwrap();
        self.cs.set_high();

        // Verify write enable was set
        Timer::after(Duration::from_micros(10)).await;
    }

    /// Send write disable command
    pub async fn write_disable(&mut self) {
        self.cs.set_low();
        self.spi.write_bytes(&[WRITE_DISABLE]).unwrap();
        self.cs.set_high();
    }

    /// Read data from flash memory
    pub async fn read_data(&mut self, address: u32, buffer: &mut [u8]) {
        let command = [
            READ_DATA,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.cs.set_low();
        // Send command and address
        self.spi.write_bytes(&command).unwrap();
        // Read data
        self.spi.transfer(buffer).unwrap();
        self.cs.set_high();
    }

    /// Fast read with dummy byte
    pub async fn fast_read(&mut self, address: u32, buffer: &mut [u8]) {
        let command = [
            FAST_READ,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
            0x00, // Dummy byte
        ];

        self.cs.set_low();
        // Send command, address, and dummy byte
        self.spi.write_bytes(&command).unwrap();
        // Read data
        self.spi.transfer(buffer).unwrap();
        self.cs.set_high();
    }

    /// Write data to flash memory (page program)
    pub async fn write_data(&mut self, address: u32, data: &[u8]) {
        // Ensure we don't cross page boundaries
        let page_offset = address as usize % PAGE_SIZE;
        let max_write_size = PAGE_SIZE - page_offset;
        let write_size = core::cmp::min(data.len(), max_write_size);

        self.wait_ready().await;
        self.write_enable().await;

        let command = [
            PAGE_PROGRAM,
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.cs.set_low();
        // Send command and address
        self.spi.write_bytes(&command).unwrap();
        // Send data
        self.spi.write_bytes(&data[..write_size]).unwrap();
        self.cs.set_high();

        // Wait for programming to complete
        self.wait_ready().await;
    }

    /// Erase 4KB sector
    pub async fn erase_sector(&mut self, address: u32) {
        self.wait_ready().await;
        self.write_enable().await;

        let command = [
            SECTOR_ERASE_4KB, // Use 0x20 for 4KB sector erase
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.cs.set_low();
        self.spi.write_bytes(&command).unwrap();
        self.cs.set_high();

        // Wait for erase to complete
        self.wait_ready().await;
    }

    /// Erase 64KB block
    pub async fn erase_block_64kb(&mut self, address: u32) {
        self.wait_ready().await;
        self.write_enable().await;

        let command = [
            BLOCK_ERASE_64KB, // Use 0xD8 for 64KB block erase
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.cs.set_low();
        self.spi.write_bytes(&command).unwrap();
        self.cs.set_high();

        // Wait for erase to complete (64KB takes longer - up to 2000ms)
        self.wait_ready().await;
    }

    /// Erase 32KB block
    pub async fn erase_block_32kb(&mut self, address: u32) {
        self.wait_ready().await;
        self.write_enable().await;

        let command = [
            BLOCK_ERASE_32KB, // Use 0x52 for 32KB block erase
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
        ];

        self.cs.set_low();
        self.spi.write_bytes(&command).unwrap();
        self.cs.set_high();

        // Wait for erase to complete
        self.wait_ready().await;
    }

    /// Erase entire chip
    pub async fn erase_chip(&mut self) {
        self.wait_ready().await;
        self.write_enable().await;

        self.cs.set_low();
        self.spi.write_bytes(&[CHIP_ERASE]).unwrap();
        self.cs.set_high();

        // Wait for erase to complete (this can take a very long time - up to 200 seconds)
        self.wait_ready().await;
    }

    /// Enter power-down mode
    pub async fn power_down(&mut self) {
        self.cs.set_low();
        self.spi.write_bytes(&[POWER_DOWN]).unwrap();
        self.cs.set_high();
    }

    /// Release from power-down mode
    pub async fn release_power_down(&mut self) {
        self.cs.set_low();
        self.spi.write_bytes(&[RELEASE_POWER_DOWN]).unwrap();
        self.cs.set_high();
    }

    /// Software reset sequence
    pub async fn software_reset(&mut self) {
        // Enable reset
        self.cs.set_low();
        self.spi.write_bytes(&[ENABLE_RESET]).unwrap();
        self.cs.set_high();

        // Reset device
        self.cs.set_low();
        self.spi.write_bytes(&[RESET_DEVICE]).unwrap();
        self.cs.set_high();

        // Wait for reset to complete
        Timer::after(Duration::from_micros(30)).await;
    }

    /// Read a single byte
    pub async fn read_byte(&mut self, address: u32) -> u8 {
        let mut buffer = [0u8; 1];
        self.read_data(address, &mut buffer).await;
        buffer[0]
    }

    /// Write a single byte
    pub async fn write_byte(&mut self, address: u32, data: u8) {
        self.write_data(address, &[data]).await;
    }

    /// Get device capacity in bytes
    pub fn capacity(&self) -> u32 {
        16 * 1024 * 1024 // 16MB for W25Q128FV
    }

    /// Get page size
    pub fn page_size(&self) -> usize {
        PAGE_SIZE
    }

    /// Get sector size
    pub fn sector_size(&self) -> usize {
        SECTOR_SIZE
    }
}
