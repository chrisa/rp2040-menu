use alloc::rc::Rc;
use embassy_time::Timer;
use slint::{
    platform::{
        software_renderer::{self, MinimalSoftwareWindow},
    },
};

use crate::display::Display;

pub type TargetPixelType = software_renderer::Rgb565Pixel;

pub mod controller;
pub mod backend;

#[embassy_executor::task()]
pub async fn render_loop(
    window: Rc<MinimalSoftwareWindow>,
    display: &'static mut Display<'static>,
) {
    loop {
        slint::platform::update_timers_and_animations();

        // blocking render
        let is_dirty = window.draw_if_needed(|renderer| {
            renderer.render(display.borrow_framebuffer_mut(), crate::display::WIDTH);
        });

        if is_dirty {
            display.draw().await.expect("drawing to display");
        } else {
            Timer::after_millis(10).await
        }
    }
}
