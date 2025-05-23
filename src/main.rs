#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use rp2040_boot2::BOOT_LOADER_W25Q080_TOP64K;
use rp2040_hal::{
    clocks::init_clocks_and_plls,
    entry,
    gpio::{FunctionSioOutput, FunctionSpi},
    pac,
    sio::Sio,
    timer::Timer,
    watchdog::Watchdog,
};

use config::CONFIG_ILI9341;

mod boot;
mod config;
mod flash;
mod sd;
mod tft;
mod uf2;
mod ui;

#[link_section = ".boot2"]
#[no_mangle]
pub static BOOT2_FIRMWARE: [u8; 256] = BOOT_LOADER_W25Q080_TOP64K;

#[link_section = ".config"]
#[no_mangle]
pub static CONFIG: [u8; 256] = CONFIG_ILI9341;

const XIP_BASE: u32 = 0x10000000;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let mut tft = tft::Tft::new(
        &mut pac.RESETS,
        pac.SPI0,
        timer.clone(),
        pins.gpio19.into_function::<FunctionSpi>(),
        pins.gpio18.into_function::<FunctionSpi>(),
        pins.gpio17.into_function::<FunctionSioOutput>(),
        pins.gpio20.into_function::<FunctionSioOutput>(),
        pins.gpio21.into_function::<FunctionSioOutput>(),
        pins.gpio22.into_function::<FunctionSioOutput>(),
    );
    tft.backlight(true);

    let mut sd = sd::SpiSD::new(
        &mut pac.RESETS,
        pac.SPI1,
        timer.clone(),
        pins.gpio11.reconfigure(),
        pins.gpio12.reconfigure(),
        pins.gpio10.reconfigure(),
        pins.gpio13.into_push_pull_output(),
    );

    // let mut y = 60;
    // sd.iterate_root_dir(|entry| {
    //     Tft.println(core::str::from_utf8(entry.name.base_name()).unwrap(), 40, y);
    //     y += 20;
    // })
    // .unwrap();

    // Set up UI
    let mut ui = ui::UI::new(tft);
    ui.init();

    let mut fw = flash::FlashWriter::new();
    uf2::read_blocks(&mut sd, "ARCADE.UF2", |block| {
        fw.next_block(block);
    });

    boot::boot(XIP_BASE + 0x100);
}
