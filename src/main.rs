#![no_std]
#![no_main]

use alloc::boxed::Box;
use assign_resources::assign_resources;
use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::peripherals;
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
use crate::ui::PicoBackend;

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
    }
}


#[global_allocator]
static HEAP: Heap = Heap::empty();

static TFT: StaticCell<Display<'_>> = StaticCell::new();
// static SD: StaticCell<SpiSD<'_>> = StaticCell::new();

static HEAP_SIZE: usize = (FRAME_SIZE * 2) + 10240;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }


    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    let window = MinimalSoftwareWindow::new(RepaintBufferType::SwappedBuffers);
    window.set_size(slint::PhysicalSize::new(display::WIDTH as u32, display::HEIGHT as u32));
    let backend = Box::new(PicoBackend::new(window.clone()));
    slint::platform::set_platform(backend).expect("backend already initialized");

    let display: &mut Display<'_> = TFT.init(display::Display::new(r.display).await);
    display.backlight(true).await;

    spawner.spawn(render_loop(window, display)).unwrap();

    let app_window = AppWindow::new().unwrap();
    app_window.show().expect("unable to show main window");

    // run the controller event loop
    // let mut controller = Controller::new(&app_window, hardware);
    // controller.run().await;

    // let sd = match sd::SpiSD::new(r.sd) {
    //     Err(_e) => panic!("failed to read card"),
    //     Ok(sd) => SD.init(sd),
    // };

    // let mut fw = flash::FlashWriter::new();
    // uf2::read_blocks(sd, "ARCADE.UF2", |block| {
    //     fw.next_block(block);
    // });

    // boot::boot(XIP_BASE + 0x100);
}
