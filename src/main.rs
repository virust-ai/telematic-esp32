#![no_std]
#![no_main]

use atat::{ResponseSlot, UrcChannel};
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{
    gpio::Output,
    prelude::*,
    rng::Trng,
    timer::timg::TimerGroup,
    twai::{self, TwaiMode},
    uart::Uart,
};
#[cfg(feature = "wdg")]
use esp_hal::rtc_cntl::{Rtc, RwdtStage};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use esp_wifi::{
    init,
    wifi::{WifiDevice, WifiStaDevice},
    EspWifiController,
};

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[derive(Debug)]
#[allow(dead_code)]
struct CanFrame {
    id: u32,
    len: u8,
    data: [u8; 8],
}

type TwaiOutbox = Channel<NoopRawMutex, CanFrame, 16>;

mod dns;
mod at_command;
mod tasks;
use static_cell::StaticCell;
use tasks::{
    can_receiver, connection, mqtt_handler, net_task, quectel_rx_handler, quectel_tx_handler,
};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    esp_alloc::heap_allocator!(72 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);
    let trng = &mut *mk_static!(Trng<'static>, Trng::new(peripherals.RNG, peripherals.ADC1));
    // let mut trng = Trng::new(peripherals.RNG, peripherals.ADC1);
    // let mut rng = Rng::new(peripherals.RNG);
    let init = &*mk_static!(
        EspWifiController<'static>,
        init(timg0.timer0, trng.rng, peripherals.RADIO_CLK).unwrap()
    );
    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(init, wifi, WifiStaDevice).unwrap();
    let config = embassy_net::Config::dhcpv4(Default::default());
    #[cfg(feature = "wdg")]
    let mut rtc = { 
        let mut rtc = Rtc::new(peripherals.LPWR);
        rtc.rwdt.enable();
        rtc.rwdt.set_timeout(RwdtStage::Stage0, 5.secs());
        rtc
    };

    let seed = 1234;

    // Init network stack
    let stack = &*mk_static!(
        Stack<WifiDevice<'_, WifiStaDevice>>,
        Stack::new(
            wifi_interface,
            config,
            mk_static!(StackResources<3>, StackResources::<3>::new()),
            seed
        )
    );

    let uart_tx_pin = peripherals.GPIO23;
    let uart_rx_pin = peripherals.GPIO15;
    let quectel_pen_pin = Output::new(peripherals.GPIO21, esp_hal::gpio::Level::High);
    let quectel_dtr_pin = Output::new(peripherals.GPIO22, esp_hal::gpio::Level::High);

    let uart0 = Uart::new(peripherals.UART0, uart_rx_pin, uart_tx_pin)
        .unwrap()
        .into_async();
    let (uart_rx, uart_tx) = uart0.split();
    static RES_SLOT: ResponseSlot<1024> = ResponseSlot::new();
    static URC_CHANNEL: UrcChannel<at_command::common::Urc, 128, 3> = UrcChannel::new();
    static INGRESS_BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let ingress = atat::Ingress::new(
        atat::AtDigester::<at_command::common::Urc>::default(),
        INGRESS_BUF.init([0; 1024]),
        &RES_SLOT,
        &URC_CHANNEL,
    );
    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let client = atat::asynch::Client::new(
        uart_tx,
        &RES_SLOT,
        BUF.init([0; 1024]),
        atat::Config::default(),
    );

    let can_tx_pin = peripherals.GPIO1;
    let can_rx_pin = peripherals.GPIO10;
    const CAN_BAUDRATE: twai::BaudRate = twai::BaudRate::B250K;
    let mut twai_config = twai::TwaiConfiguration::new(
        peripherals.TWAI0,
        can_rx_pin,
        can_tx_pin,
        CAN_BAUDRATE,
        TwaiMode::Normal,
    )
    .into_async();
    twai_config.set_filter(
        const {
            twai::filter::SingleStandardFilter::new(
                b"xxxxxxxxxxx",
                b"x",
                [b"xxxxxxxx", b"xxxxxxxx"],
            )
        },
    );
    let can = twai_config.start();
    static CHANNEL: StaticCell<TwaiOutbox> = StaticCell::new();
    let channel = &*CHANNEL.init(Channel::new());
    let (can_rx, _can_tx) = can.split();

    spawner.spawn(can_receiver(can_rx, channel)).ok();
    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(stack)).ok();
    spawner.spawn(mqtt_handler(stack, trng, channel)).ok();
    spawner.spawn(quectel_rx_handler(ingress, uart_rx)).ok();
    spawner
        .spawn(quectel_tx_handler(client, quectel_pen_pin, quectel_dtr_pin))
        .ok();
    loop {
        Timer::after_secs(2).await;
        #[cfg(feature = "wdg")]
        rtc.rwdt.feed();
    }
}
