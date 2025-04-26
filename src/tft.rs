use defmt_rtt as _;
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
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use rp_pico::hal::{
    gpio::{FunctionSpi, Pin, PinId, PullDown},
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

type TFTSpiDevice<S, Tx, Sck, Cs> = ExclusiveDevice<TFTSpi<S, Tx, Sck>, Cs, NoDelay>;

type TFTSpiInterface<S, Tx, Sck, Cs, Dc> = SPIInterface<TFTSpiDevice<S, Tx, Sck, Cs>, Dc>;

pub struct TFT<S, Tx, Sck, Cs, Dc, Rst, Bl>
where
    S: SpiDevice,
    Tx: PinId + ValidPinIdTx<S>,
    Sck: PinId + ValidPinIdSck<S>,
    Cs: OutputPin,
    Dc: OutputPin,
{
    display: Ili9341<TFTSpiInterface<S, Tx, Sck, Cs, Dc>, Rst>,
    backlight: Bl,
}

impl<S, Tx, Sck, Cs, Dc, Rst, Bl> TFT<S, Tx, Sck, Cs, Dc, Rst, Bl>
where
    S: SpiDevice,
    Tx: PinId + ValidPinIdTx<S>,
    Sck: PinId + ValidPinIdSck<S>,
    Cs: OutputPin,
    Dc: OutputPin,
    Rst: OutputPin,
    Bl: OutputPin,
{
    pub fn new(
        spi_device: TFTSpiDevice<S, Tx, Sck, Cs>,
        dc: Dc,
        rst: Rst,
        backlight: Bl,
        delay: &mut Timer,
    ) -> TFT<S, Tx, Sck, Cs, Dc, Rst, Bl> {
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
