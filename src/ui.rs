use embedded_graphics::{
    mono_font::{iso_8859_1::FONT_6X9, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use embedded_layout::prelude::*;

pub struct UI {
    tft: crate::tft::Tft,
}

impl UI {
    pub fn new(tft: crate::tft::Tft) -> UI {
        Self { tft }
    }

    pub fn init(&mut self) {
        self.tft.clear(Rgb565::WHITE);
        // self.tft.println("Hello from RP2040", 100, 40);

        let display_area = self.tft.display.bounding_box();

        let text_style = MonoTextStyle::new(&FONT_6X9, Rgb565::RED);

        Text::new("Loading...", Point::zero(), text_style)
            // align text to the center of the display
            .align_to(&display_area, horizontal::Center, vertical::Center)
            .draw(&mut self.tft.display)
            .unwrap();
    }
}
