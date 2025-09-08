use alloc::rc::Rc;
use embassy_time::Instant;
use slint::{
    PlatformError,
    platform::{
        Platform, WindowAdapter,
        software_renderer::MinimalSoftwareWindow,
    },
};

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
