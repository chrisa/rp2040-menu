use crate::tft::Tft;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};

pub struct UI<'spi> {
    tft: &'spi mut Tft<'spi>,
}

impl<'spi> UI<'spi> {
    pub fn new(tft: &'spi mut Tft<'spi>) -> UI<'spi> {
        Self { tft }
    }

    pub async fn init(&mut self) {
        self.tft.clear(Rgb565::WHITE).await;
        // self.tft.test_image().await;

        self.tft
            .draw(|display| {
                // Create styles used by the drawing operations.
                let thin_stroke = PrimitiveStyle::with_stroke(Rgb565::BLUE, 1);
                let thick_stroke = PrimitiveStyle::with_stroke(Rgb565::RED, 3);
                let border_stroke = PrimitiveStyleBuilder::new()
                    .stroke_color(Rgb565::RED)
                    .stroke_width(3)
                    .stroke_alignment(StrokeAlignment::Inside)
                    .build();
                let fill = PrimitiveStyle::with_fill(Rgb565::GREEN);
                let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::BLACK);

                let yoffset = 10;

                // Draw a 3px wide outline around the display.
                display
                    .bounding_box()
                    .into_styled(border_stroke)
                    .draw(display)?;

                // Draw a triangle.
                Triangle::new(
                    Point::new(16, 16 + yoffset),
                    Point::new(16 + 16, 16 + yoffset),
                    Point::new(16 + 8, yoffset),
                )
                .into_styled(thin_stroke)
                .draw(display)?;

                // Draw a filled square
                Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
                    .into_styled(fill)
                    .draw(display)?;

                // Draw a circle with a 3px wide stroke.
                Circle::new(Point::new(88, yoffset), 17)
                    .into_styled(thick_stroke)
                    .draw(display)?;

                // Draw centered text.
                let text = "embedded-graphics";
                Text::with_alignment(
                    text,
                    display.bounding_box().center() + Point::new(0, 15),
                    character_style,
                    Alignment::Center,
                )
                .draw(display)?;

                Ok(())
            })
            .await
            .expect("ui");
    }
}
