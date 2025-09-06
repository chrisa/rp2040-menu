// mod backend;
// mod renderer;

use alloc::rc::Rc;
use embassy_time::{Instant, Timer};
use slint::{
    platform::{
        software_renderer::{self, MinimalSoftwareWindow},
        Platform, WindowAdapter,
    },
    PlatformError,
};

use crate::display::Display;

pub type TargetPixelType = software_renderer::Rgb565Pixel;


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
            display.draw().await.unwrap();
        } else {
            Timer::after_millis(10).await
        }
    }
}


pub struct PicoBackend {
    window: Rc<MinimalSoftwareWindow>,
}

impl PicoBackend {
    pub fn new(window: Rc<MinimalSoftwareWindow>) -> Self {
        Self { window }
    }
}

impl Platform for PicoBackend {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        let window = self.window.clone();
        Ok(window)
    }

    fn duration_since_start(&self) -> core::time::Duration {
        Instant::now().duration_since(Instant::from_secs(0)).into()
    }
}