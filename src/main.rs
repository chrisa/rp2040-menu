//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use fugit::RateExtU32;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, watchdog::Watchdog};

use display_interface_spi::SPIInterface;
// use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Triangle},
    text::{Alignment, Text},
};
use embedded_hal::spi::MODE_0;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use bsp::hal::{
    gpio::{
        bank0::{Gpio17, Gpio18, Gpio19, Gpio20, Gpio21},
        FunctionSioOutput, FunctionSpi, Pin, PullDown,
    },
    pac::SPI0,
    spi::{Enabled, Spi},
    timer::Timer,
};

type TFTSpi = Spi<
    Enabled,
    SPI0,
    (
        Pin<Gpio19, FunctionSpi, PullDown>,
        Pin<Gpio18, FunctionSpi, PullDown>,
    ),
>;
type TFTSpiDevice = ExclusiveDevice<TFTSpi, Pin<Gpio17, FunctionSioOutput, PullDown>, NoDelay>;
type TFTSpiInterface = SPIInterface<TFTSpiDevice, Pin<Gpio20, FunctionSioOutput, PullDown>>;

pub struct TFT {
    display: Ili9341<TFTSpiInterface, Pin<Gpio21, FunctionSioOutput, PullDown>>,
}

impl TFT {
    pub fn new(
        spi: pac::SPI0,
        cs: Pin<Gpio17, FunctionSioOutput, PullDown>,
        sclk: Pin<Gpio18, FunctionSpi, PullDown>,
        mosi: Pin<Gpio19, FunctionSpi, PullDown>,
        dc: Pin<Gpio20, FunctionSioOutput, PullDown>,
        rst: Pin<Gpio21, FunctionSioOutput, PullDown>,
        resets: &mut pac::RESETS,
        delay: &mut Timer,
    ) -> TFT {
        let spi_pin_layout = (mosi, sclk);
        let spi = Spi::<_, _, _, 8>::new(spi, spi_pin_layout).init(
            resets,
            125_000_000u32.Hz(),
            16_000_000u32.Hz(),
            MODE_0,
        );

        let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
        let interface = SPIInterface::new(spi_device, dc);

        let display = Ili9341::new(
            interface,
            rst,
            delay,
            Orientation::Portrait,
            DisplaySize240x320,
        )
        .unwrap();

        TFT { display }
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.display.clear(color).unwrap();
    }

    pub fn part_clear(&mut self, x: i32, y: i32, w: u32, h: u32) {
        Rectangle::new(Point::new(x, y), Size::new(w, h))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn println(&mut self, text: &str, x: i32, y: i32) {
        let style = MonoTextStyle::new(&FONT_8X13, Rgb565::RED);
        Text::with_alignment(text, Point::new(x, y), style, Alignment::Center)
            .draw(&mut self.display)
            .unwrap();
    }
}

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    // let core = pac::CorePeripherals::take().unwrap();
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

    // let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let cs = pins.gpio17.into_function::<FunctionSioOutput>();
    let sclk = pins.gpio18.into_function::<FunctionSpi>();
    let mosi = pins.gpio19.into_function::<FunctionSpi>();
    let dc = pins.gpio20.into_function::<FunctionSioOutput>();
    let rst = pins.gpio21.into_function::<FunctionSioOutput>();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut tft = TFT::new(
        pac.SPI0,
        cs,
        sclk,
        mosi,
        dc,
        rst,
        &mut pac.RESETS,
        &mut timer,
    );
    tft.clear(Rgb565::WHITE);
    tft.println("Hello from RP2040", 100, 40);

    loop {
        // your business logic
    }
}

// End of file
