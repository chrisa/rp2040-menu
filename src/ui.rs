use crate::tft::Tft;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

pub struct UI<'spi> {
    tft: &'spi mut Tft<'spi>,
}

impl<'spi> UI<'spi> {
    pub fn new(tft: &'spi mut Tft<'spi>) -> UI<'spi> {
        Self { tft }
    }

    pub async fn init(&mut self) {
        self.tft.clear(Rgb565::WHITE).await;
        self.tft.test_image().await;

        // self.tft.println("Hello from RP2040", 100, 40);

        // let display_area = self.tft.display.bounding_box();

        // let text_style = MonoTextStyle::new(&FONT_6X9, Rgb565::RED);

        // Text::new("Loading...", Point::zero(), text_style)
        //     // align text to the center of the display
        //     .align_to(&display_area, horizontal::Center, vertical::Center)
        //     .draw(&mut self.tft.display)
        //     .unwrap();
    }
}
