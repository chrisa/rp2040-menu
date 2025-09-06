use core::convert::Infallible;

use defmt_rtt as _;
use embassy_time::Delay;
use embedded_hal_02::spi::MODE_0;
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_probe as _;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use embassy_rp::{
    gpio::{Level, Output},
    peripherals::SPI0,
    spi::{Async, Config as SpiConfig, Spi},
};
use lcd_async::{
    Builder, Display as LcdDisplay, interface,
    models::ILI9342CRgb565,
    options::{ColorInversion, Orientation, Rotation},
};
use slint::platform::software_renderer::{Rgb565Pixel, TargetPixel};
use static_cell::StaticCell;

use crate::{ui::TargetPixelType, DisplayResources};

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

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const FRAME_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize);

type SpiInterface<'spi> = interface::SpiInterface<
    ExclusiveDevice<Spi<'spi, SPI0, Async>, Output<'spi>, Delay>,
    Output<'spi>,
>;

type SpiDisplay<'spi> = LcdDisplay<SpiInterface<'spi>, ILI9342CRgb565, Output<'spi>>;

pub type DisplayError = interface::SpiError<
    embedded_hal_bus::spi::DeviceError<embassy_rp::spi::Error, Infallible>,
    Infallible,
>;

pub struct Display<'spi> {
    display: SpiDisplay<'spi>,
    backlight: Output<'spi>,
    framebuffer: &'spi mut Box<[TargetPixelType; FRAME_SIZE]>,
}

static FB: StaticCell<Box<[TargetPixelType; FRAME_SIZE]>> = StaticCell::new();

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
            .display_size(WIDTH as u16, HEIGHT as u16)
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

        let framebuffer: &'static mut Box<[Rgb565Pixel; _]> = FB.init(box_array! [Rgb565Pixel::from_rgb(255u8, 0u8, 0u8); FRAME_SIZE]);

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

    pub async fn draw(
        &mut self,
    ) -> Result<(), DisplayError> {
        let ptr = self.framebuffer.as_ptr().cast::<[u8; FRAME_SIZE * 2]>();
        self.display
            .show_raw_data(0, 0, WIDTH as u16, HEIGHT as u16, unsafe { &*ptr })
            .await?;
        Ok(())
    }

    pub fn borrow_framebuffer_mut(&mut self) -> &mut [TargetPixelType; FRAME_SIZE] {
        self.framebuffer
    }
}
