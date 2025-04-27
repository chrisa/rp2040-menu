#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use fugit::RateExtU32;
use panic_probe as _;
use rp2040_boot2::BOOT_LOADER_W25Q080_TOP64K;
use rp2040_hal::entry;

use embedded_hal::spi::MODE_0;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use rp2040_hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionSioOutput, FunctionSpi, Pin, PullNone, PullUp},
    pac,
    sio::Sio,
    spi::Spi,
    timer::Timer,
    watchdog::Watchdog,
};

mod sd;
mod tft;

#[link_section = ".boot2"]
#[no_mangle]
pub static BOOT2_FIRMWARE: [u8; 256] = BOOT_LOADER_W25Q080_TOP64K;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut tft;

    {
        let cs = pins.gpio17.into_function::<FunctionSioOutput>();
        let sclk = pins.gpio18.into_function::<FunctionSpi>();
        let mosi = pins.gpio19.into_function::<FunctionSpi>();
        let dc = pins.gpio20.into_function::<FunctionSioOutput>();
        let rst = pins.gpio21.into_function::<FunctionSioOutput>();
        let backlight = pins.gpio22.into_function::<FunctionSioOutput>();

        let spi_pin_layout = (mosi, sclk);
        let spi = Spi::<_, _, _, 8>::new(pac.SPI0, spi_pin_layout).init(
            &mut pac.RESETS,
            125u32.MHz(),
            64000u32.kHz(),
            MODE_0,
        );

        let spi_device =
            ExclusiveDevice::new(spi, cs, NoDelay).expect("failed to create TFT SPI device");
        tft = tft::TFT::new(spi_device, dc, rst, backlight, &mut timer);

        tft.backlight(true);
        tft.clear(Rgb565::WHITE);
        tft.println("Hello from RP2040", 100, 40);
    }

    {
        let cs = pins.gpio13.into_push_pull_output();
        let sclk: Pin<_, FunctionSpi, PullNone> = pins.gpio10.reconfigure();
        let mosi: Pin<_, FunctionSpi, PullNone> = pins.gpio11.reconfigure();
        let miso: Pin<_, FunctionSpi, PullUp> = pins.gpio12.reconfigure();

        let spi_pin_layout = (mosi, miso, sclk);

        let spi = Spi::<_, _, _, 8>::new(pac.SPI1, spi_pin_layout).init(
            &mut pac.RESETS,
            125u32.MHz(),
            400u32.kHz(),
            MODE_0,
        );

        let spi_device =
            ExclusiveDevice::new(spi, cs, NoDelay).expect("failed to create SD SPI dev");

        let sd = sd::SpiSD::new(spi_device, timer);
        let mut y = 60;
        sd.iterate_root_dir(|entry| {
            tft.println(core::str::from_utf8(entry.name.base_name()).unwrap(), 40, y);
            y += 20;
        })
        .unwrap();
    }

    loop {
        // your business logic
    }
}

// End of file
