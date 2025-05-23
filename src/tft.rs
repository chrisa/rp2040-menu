use defmt_rtt as _;
use fugit::RateExtU32;
use panic_probe as _;

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use ili9341::{DisplaySize240x320, Ili9341, Orientation, SPI_MODE};

use rp2040_hal::{
    gpio::{
        bank0::{Gpio17, Gpio18, Gpio19, Gpio20, Gpio21, Gpio22},
        FunctionSioOutput, FunctionSpi, Pin, PullDown,
    },
    pac::{self, SPI0},
    spi::{Enabled, Spi},
};

pub struct Tft {
    pub display: Ili9341<
        SPIInterface<
            ExclusiveDevice<
                Spi<
                    Enabled,
                    SPI0,
                    (
                        Pin<Gpio19, FunctionSpi, PullDown>,
                        Pin<Gpio18, FunctionSpi, PullDown>,
                    ),
                >,
                Pin<Gpio17, rp2040_hal::gpio::FunctionSio<rp2040_hal::gpio::SioOutput>, PullDown>,
                NoDelay,
            >,
            Pin<Gpio20, rp2040_hal::gpio::FunctionSio<rp2040_hal::gpio::SioOutput>, PullDown>,
        >,
        Pin<Gpio21, rp2040_hal::gpio::FunctionSio<rp2040_hal::gpio::SioOutput>, PullDown>,
    >,
    backlight: Pin<Gpio22, FunctionSioOutput, PullDown>,
}

impl Tft {
    pub fn new(
        resets: &mut rp2040_hal::pac::RESETS,
        spi: pac::SPI0,
        mut timer: rp2040_hal::timer::Timer,
        mosi: Pin<Gpio19, FunctionSpi, PullDown>,
        sclk: Pin<Gpio18, FunctionSpi, PullDown>,
        cs: Pin<Gpio17, FunctionSioOutput, PullDown>,
        dc: Pin<Gpio20, FunctionSioOutput, PullDown>,
        rst: Pin<Gpio21, FunctionSioOutput, PullDown>,
        backlight: Pin<Gpio22, FunctionSioOutput, PullDown>,
    ) -> Tft {
        let spi_pin_layout = (mosi, sclk);
        let spi = Spi::<_, _, _, 8>::new(spi, spi_pin_layout).init(
            resets,
            125u32.MHz(),
            64000u32.kHz(),
            SPI_MODE,
        );

        let spi_device =
            ExclusiveDevice::new(spi, cs, NoDelay).expect("failed to create Tft SPI device");

        let interface = SPIInterface::new(spi_device, dc);

        let display = Ili9341::new(
            interface,
            rst,
            &mut timer,
            Orientation::Landscape,
            DisplaySize240x320,
        )
        .unwrap();

        Tft { display, backlight }
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
