use core::convert::Infallible;

use defmt_rtt as _;
use embassy_time::Delay;
use embedded_hal_02::spi::MODE_0;
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_probe as _;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use embedded_graphics::{Drawable, draw_target::DrawTarget, pixelcolor::Rgb565};

use embassy_rp::{
    gpio::{Level, Output},
    peripherals::SPI0,
    spi::{Async, Config as SpiConfig, Spi},
};
use lcd_async::{
    Builder, Display as LcdDisplay, TestImage, interface,
    models::ILI9342CRgb565,
    options::{ColorInversion, Orientation, Rotation},
    raw_framebuf::RawFrameBuf,
};
use static_cell::StaticCell;

use crate::DisplayResources;

macro_rules! box_array {
    ($val:expr ; $len:expr) => {{
        // Use a generic function so that the pointer cast remains type-safe
        fn vec_to_boxed_array<T>(vec: Vec<T>) -> Box<[T; $len]> {
            let boxed_slice = vec.into_boxed_slice();

            let ptr = Box::into_raw(boxed_slice) as *mut [T; $len];

            unsafe { Box::from_raw(ptr) }
        }

        vec_to_boxed_array(vec![$val; $len])
    }};
}

const WIDTH: u16 = 320;
const HEIGHT: u16 = 240;
const PIXEL_SIZE: usize = 2; // RGB565 = 2 bytes per pixel
pub const FRAME_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) * PIXEL_SIZE;

type SpiInterface<'spi> = interface::SpiInterface<
    ExclusiveDevice<Spi<'spi, SPI0, Async>, Output<'spi>, Delay>,
    Output<'spi>,
>;

type SpiDisplay<'spi> = LcdDisplay<SpiInterface<'spi>, ILI9342CRgb565, Output<'spi>>;

type DisplayError = interface::SpiError<
    embedded_hal_bus::spi::DeviceError<embassy_rp::spi::Error, Infallible>,
    Infallible,
>;

type DisplayBuffer<'a> = RawFrameBuf<Rgb565, &'a mut [u8]>;

pub struct Display<'spi> {
    display: SpiDisplay<'spi>,
    backlight: Output<'spi>,
    framebuffer: &'spi mut Box<[u8; FRAME_SIZE]>,
}

static FB: StaticCell<Box<[u8; FRAME_SIZE]>> = StaticCell::new();

impl<'spi> Display<'spi> {
    pub async fn new(res: DisplayResources) -> Display<'spi> {
        let mut spi_cfg = SpiConfig::default();
        spi_cfg.frequency = 125_000_000;
        spi_cfg.polarity = MODE_0.polarity;
        spi_cfg.phase = MODE_0.phase;
        let spi = Spi::new_txonly(res.spi, res.sclk, res.mosi, res.dma, spi_cfg);

        let cs = Output::new(res.cs, Level::Low);
        let dc = Output::new(res.dc, Level::Low);
        let rst = Output::new(res.rst, Level::Low);

        let spi_delay = embassy_time::Delay;
        let spi_device = ExclusiveDevice::new(spi, cs, spi_delay)
            .expect("failed to create exclusive bus for display");
        let di = interface::SpiInterface::new(spi_device, dc);

        let mut delay = embassy_time::Delay;
        let display = Builder::new(ILI9342CRgb565, di)
            .reset_pin(rst)
            .display_size(WIDTH, HEIGHT)
            .invert_colors(ColorInversion::Normal)
            .orientation(Orientation {
                rotation: Rotation::Deg90,
                mirrored: true,
            })
            .display_offset(0, 0)
            .init(&mut delay)
            .await
            .unwrap();

        let backlight = Output::new(res.backlight, Level::Low);

        let framebuffer = FB.init(box_array! [0; FRAME_SIZE]);

        Display {
            display,
            backlight,
            framebuffer,
        }
    }

    pub async fn backlight(&mut self, on: bool) {
        if on {
            self.backlight.set_high()
        } else {
            self.backlight.set_low();
        }
    }

    pub async fn clear(&mut self, color: Rgb565) {
        self.draw(|raw_fb| {
            raw_fb.clear(color).unwrap();
            Ok(())
        })
        .await
        .expect("clear");
    }

    pub async fn test_image(&mut self) {
        self.draw(|raw_fb| {
            let test: TestImage<Rgb565> = TestImage::new();
            test.draw(raw_fb)?;
            Ok(())
        })
        .await
        .expect("test image");
    }

    pub async fn draw(
        &mut self,
        func: impl FnOnce(&mut DisplayBuffer) -> Result<(), Infallible>,
    ) -> Result<(), DisplayError> {
        let mut raw_fb =
            DisplayBuffer::new(self.framebuffer.as_mut_slice(), WIDTH.into(), HEIGHT.into());
        func(&mut raw_fb).expect("infallible drawing op");
        self.display
            .show_raw_data(0, 0, WIDTH, HEIGHT, self.framebuffer.as_slice())
            .await?;
        Ok(())
    }
}
