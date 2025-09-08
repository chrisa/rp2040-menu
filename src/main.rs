#![no_std]
#![no_main]

use alloc::boxed::Box;
use assign_resources::assign_resources;
use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::gpio::Input;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Duration;
use embassy_time::Timer;
use slint::platform::software_renderer::MinimalSoftwareWindow;
use slint::platform::software_renderer::RepaintBufferType;
use static_cell::StaticCell;

use defmt_rtt as _;
use panic_probe as _;

use config::CONFIG_ILI9341;
use rp2040_boot2::BOOT_LOADER_W25Q080_TOP64K;

use crate::display::Display;
use crate::display::FRAME_SIZE;
use crate::sd::SpiSD;
use crate::ui::backend::PicoBackend;
use crate::ui::controller::ButtonEvent;

use core::ptr::addr_of_mut;

extern crate alloc;

use embedded_alloc::LlffHeap as Heap;

mod boot;
mod config;
mod display;
mod flash;
mod sd;
mod uf2;
mod ui;

use crate::ui::controller::Controller;
use crate::ui::render_loop;

slint::include_modules!();

#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
pub static BOOT2_FIRMWARE: [u8; 256] = BOOT_LOADER_W25Q080_TOP64K;

#[unsafe(link_section = ".config")]
#[unsafe(no_mangle)]
pub static CONFIG: [u8; 256] = CONFIG_ILI9341;

const XIP_BASE: u32 = 0x10000000;

assign_resources! {
    display: DisplayResources {
        spi: SPI0,
        mosi: PIN_19,
        sclk: PIN_18,
        cs: PIN_17,
        dc: PIN_20,
        rst: PIN_21,
        backlight: PIN_22,
        dma: DMA_CH0,
    },
    sd: SdResources {
        spi: SPI1,
        mosi: PIN_11,
        miso: PIN_12,
        sclk: PIN_10,
        cs: PIN_13,
        tx_dma: DMA_CH1,
        rx_dma: DMA_CH2,
    },
    buttons: ButtonResources {
        up_pin: PIN_2,
        down_pin: PIN_3,
        select_pin: PIN_4,
        refresh_pin: PIN_5,
    },
}

#[global_allocator]
static HEAP: Heap = Heap::empty();
static HEAP_SIZE: usize = (FRAME_SIZE * 2) + 32768;

static BUTTON_SIGNAL: Signal<ThreadModeRawMutex, ButtonEvent> = Signal::new();

static TFT: StaticCell<Display<'_>> = StaticCell::new();
static SD: StaticCell<SpiSD<'_>> = StaticCell::new();
static CONTROLLER: StaticCell<Controller<'_>> = StaticCell::new();
static UI: StaticCell<FileSelector> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    let window = MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
    window.set_size(slint::PhysicalSize::new(
        display::WIDTH as u32,
        display::HEIGHT as u32,
    ));
    let backend = Box::new(PicoBackend::new(window.clone()));
    slint::platform::set_platform(backend).expect("backend already initialized");

    let display: &mut Display<'_> = TFT.init(display::Display::new(r.display).await);
    display.backlight(true).await;

    spawner
        .spawn(render_loop(window, display))
        .expect("render_loop");

    let ui: &'static FileSelector = UI.init(FileSelector::new().expect("fileselector"));
    ui.show().expect("unable to show main window");

    let sd: &'static SpiSD<'_> = match sd::SpiSD::new(r.sd) {
        Err(e) => panic!("failed to read card: {:?}", e),
        Ok(sd) => SD.init(sd),
    };

    let controller = CONTROLLER.init(Controller::new(ui, sd).expect("controller"));

    spawner.spawn(ui_task(controller)).expect("ui_task");
    spawner
        .spawn(button_handler(r.buttons))
        .expect("button task");
}

#[embassy_executor::task]
async fn ui_task(controller: &'static Controller<'static>) {
    // Initial file load
    controller.refresh_files().await;

    loop {
        // Wait for button event
        let button_event = BUTTON_SIGNAL.wait().await;

        match button_event {
            ButtonEvent::Refresh => {
                controller.refresh_files().await;
            }
            ButtonEvent::Select => {
                controller.handle_button(button_event);
            }
            _ => {
                controller.handle_button(button_event);
            }
        }

        Timer::after(Duration::from_millis(10)).await;
    }
}

// Button handler task
#[embassy_executor::task]
async fn button_handler(r: ButtonResources) {
    let up_button = Input::new(r.up_pin, Pull::Up);
    let down_button = Input::new(r.down_pin, Pull::Up);
    let select_button = Input::new(r.select_pin, Pull::Up);
    let refresh_button = Input::new(r.refresh_pin, Pull::Up);

    loop {
        // Check each button (active low with pull-up)
        if up_button.is_low() {
            BUTTON_SIGNAL.signal(ButtonEvent::Up);
            Timer::after(Duration::from_millis(200)).await; // Debounce
        }

        if down_button.is_low() {
            BUTTON_SIGNAL.signal(ButtonEvent::Down);
            Timer::after(Duration::from_millis(200)).await;
        }

        if select_button.is_low() {
            BUTTON_SIGNAL.signal(ButtonEvent::Select);
            Timer::after(Duration::from_millis(200)).await;
        }

        if refresh_button.is_low() {
            BUTTON_SIGNAL.signal(ButtonEvent::Refresh);
            Timer::after(Duration::from_millis(200)).await;
        }

        Timer::after(Duration::from_millis(50)).await;
    }
}
