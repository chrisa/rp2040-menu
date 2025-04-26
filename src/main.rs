#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use fugit::RateExtU32;
use panic_probe as _;
use rp_pico::entry;

use rp_pico::hal::gpio::bank0::{Gpio14, Gpio15, Gpio27};
use rp_pico::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, watchdog::Watchdog};

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use embedded_hal::{digital::OutputPin, spi::MODE_0};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use rp_pico::hal::{
    gpio::{DynPinId, FunctionSioOutput, FunctionSpi, Pin, PinId, PullDown},
    spi::{Enabled, Spi, SpiDevice, ValidPinIdSck, ValidPinIdTx},
    timer::Timer,
};

type TFTSpi<S, Tx, Sck> = Spi<
    Enabled,
    S,
    (
        Pin<Tx, FunctionSpi, PullDown>,
        Pin<Sck, FunctionSpi, PullDown>,
    ),
>;
type TFTSpiDevice<S, Tx, Sck> =
    ExclusiveDevice<TFTSpi<S, Tx, Sck>, Pin<DynPinId, FunctionSioOutput, PullDown>, NoDelay>;
type TFTSpiInterface<S, Tx, Sck> =
    SPIInterface<TFTSpiDevice<S, Tx, Sck>, Pin<DynPinId, FunctionSioOutput, PullDown>>;

pub struct TFT<S, Tx, Sck>
where
    S: SpiDevice,
    Tx: PinId + ValidPinIdTx<S>,
    Sck: PinId + ValidPinIdSck<S>,
{
    display: Ili9341<TFTSpiInterface<S, Tx, Sck>, Pin<DynPinId, FunctionSioOutput, PullDown>>,
    backlight: Pin<DynPinId, FunctionSioOutput, PullDown>,
}

impl<S, Tx, Sck> TFT<S, Tx, Sck>
where
    S: SpiDevice,
    Tx: PinId + ValidPinIdTx<S>,
    Sck: PinId + ValidPinIdSck<S>,
{
    pub fn new(
        spi_device: TFTSpiDevice<S, Tx, Sck>,
        dc: Pin<DynPinId, FunctionSioOutput, PullDown>,
        rst: Pin<DynPinId, FunctionSioOutput, PullDown>,
        backlight: Pin<DynPinId, FunctionSioOutput, PullDown>,
        delay: &mut Timer,
    ) -> TFT<S, Tx, Sck> {
        let interface = SPIInterface::new(spi_device, dc);

        let display = Ili9341::new(
            interface,
            rst,
            delay,
            Orientation::Landscape,
            DisplaySize240x320,
        )
        .unwrap();

        TFT { display, backlight }
    }

    pub fn backlight(&mut self, on: bool) {
        if on {
            self.backlight.set_high().unwrap();
        } else {
            self.backlight.set_low().unwrap();
        }
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

    let pins = rp_pico::Pins::new(
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
    let backlight = pins.gpio22.into_function::<FunctionSioOutput>();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let spi_pin_layout = (mosi, sclk);
    let spi = Spi::<_, _, _, 8>::new(pac.SPI0, spi_pin_layout).init(
        &mut pac.RESETS,
        125u32.MHz(),
        64000u32.kHz(),
        MODE_0,
    );

    let spi_device = ExclusiveDevice::new_no_delay(spi, cs.into_dyn_pin()).unwrap();

    let mut tft = TFT::<_, _, _>::new(
        spi_device,
        dc.into_dyn_pin(),
        rst.into_dyn_pin(),
        backlight.into_dyn_pin(),
        &mut timer,
    );
    tft.backlight(true);
    tft.clear(Rgb565::WHITE);
    tft.println("Hello from RP2040", 100, 40);

    loop {
        // your business logic
    }
}

// End of file
