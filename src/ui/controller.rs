use alloc::rc::Rc;
use alloc::vec::Vec;
use slint::ComponentHandle;
use slint::Model;
use slint::SharedString;
use slint::format;

use crate::XIP_BASE;
use crate::boot::boot;
use crate::flash::FlashWriter;
use crate::sd::SpiSD;
use crate::slint_generatedFileSelector::FileSelector;
use crate::uf2::read_blocks;

#[derive(Clone, Copy)]
pub enum ButtonEvent {
    Up,
    Down,
    Select,
    Refresh,
}

pub struct Controller<'spi> {
    ui: &'spi FileSelector,
    sd: &'spi SpiSD<'spi>,
}

impl<'spi> Controller<'spi> {
    pub fn new(
        ui: &'spi FileSelector,
        sd: &'spi SpiSD<'spi>,
    ) -> Result<Self, slint::PlatformError> {
        let controller = Self { ui, sd };
        controller.setup_callbacks();
        Ok(controller)
    }

    fn setup_callbacks(&self) {
        // Move up callback
        let ui_weak = self.ui.as_weak();
        self.ui.on_move_up(move || {
            let ui = ui_weak.unwrap();
            let current = ui.get_selected_index();
            let file_count = ui.get_file_list().row_count() as i32;

            if file_count > 0 {
                let new_index = if current > 0 {
                    current - 1
                } else {
                    file_count - 1
                };
                ui.set_selected_index(new_index);
            }
        });

        // Move down callback
        let ui_weak = self.ui.as_weak();
        self.ui.on_move_down(move || {
            let ui = ui_weak.unwrap();
            let current = ui.get_selected_index();
            let file_count = ui.get_file_list().row_count() as i32;

            if file_count > 0 {
                let new_index = if current < file_count - 1 {
                    current + 1
                } else {
                    0
                };
                ui.set_selected_index(new_index);
            }
        });

        // Select file callback
        let ui_weak = self.ui.as_weak();
        self.ui.on_select_file(move || {
            let ui = ui_weak.unwrap();
            let selected = ui.get_selected_file();
            ui.set_status_message(format!("Selected: {}", selected));
        });

        // Refresh callback - this will be handled by the main task
        let ui_weak = self.ui.as_weak();
        self.ui.on_refresh_files(move || {
            let ui = ui_weak.unwrap();
            ui.set_status_message("Refreshing...".into());
        });
    }

    pub async fn refresh_files(&self) {
        self.ui.set_status_message("Loading files...".into());

        let files = self.sd.list_files();
        let slint_files: Vec<SharedString> = files.into_iter().map(|s| s.into()).collect();

        let model = Rc::new(slint::VecModel::from(slint_files));
        self.ui.set_file_list(model.into());
        self.ui.set_selected_index(0);
        self.ui.set_status_message("Files loaded".into());
    }

    pub fn handle_button(&self, button: ButtonEvent) {
        match button {
            ButtonEvent::Up => self.ui.invoke_move_up(),
            ButtonEvent::Down => self.ui.invoke_move_down(),
            ButtonEvent::Select => {
                self.ui.invoke_select_file();
                self.boot_selected_file();
            }
            ButtonEvent::Refresh => self.ui.invoke_refresh_files(),
        }
    }

    pub fn boot_selected_file(&self) {
        let filename = self.ui.get_selected_file();
        if filename.is_empty() {
            panic!("no selected filename?");
        }

        cortex_m::interrupt::disable();

        let mut fw = FlashWriter::new();
        read_blocks(self.sd, &filename, |block| {
            fw.next_block(block);
        });

        boot(XIP_BASE + 0x100);
    }
}
