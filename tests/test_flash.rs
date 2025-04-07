use embassy_test::block_on;

#[test]
fn test_flash_communication() {
    block_on(async {
        let peripherals = Peripherals::take().unwrap();
        let pins = peripherals.pins;

        let spi = Spi::new(
            peripherals.spi2,
            pins.gpio6,
            pins.gpio7,
            pins.gpio8,
            Config::default(),
        );
        let cs = Output::new(pins.gpio9, Level::High);

        let mut flash = W25Q128FVSG::new(spi, cs);

        flash.init().unwrap();

        let id = flash.read_id().unwrap();
        println!("JEDEC ID: {:x?}", id);

        let address = 0x0000;
        let write_data = [0xAA, 0xBB, 0xCC];
        flash.write_bytes(address, &write_data).unwrap();

        let mut read_data = [0; 3];
        flash.read_bytes(address, &mut read_data).unwrap();
        println!("Read Data: {:x?}", read_data);

        flash.erase_sector(0).unwrap();
        println!("Sector erased.");
    });
}
