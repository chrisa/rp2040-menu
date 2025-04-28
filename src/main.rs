#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use fugit::RateExtU32;
use panic_probe as _;
use rp2040_boot2::BOOT_LOADER_W25Q080_TOP64K;
use rp2040_hal::entry;

use embedded_hal::spi::MODE_0;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use rp2040_hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionSioOutput, FunctionSpi, Pin, PullNone, PullUp},
    pac,
    sio::Sio,
    spi::Spi,
    timer::Timer,
    watchdog::Watchdog,
};

mod boot;
mod flash;
mod sd;
mod tft;
mod uf2;

#[link_section = ".boot2"]
#[no_mangle]
pub static BOOT2_FIRMWARE: [u8; 256] = BOOT_LOADER_W25Q080_TOP64K;

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

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut tft;

    {
        let cs = pins.gpio17.into_function::<FunctionSioOutput>();
        let sclk = pins.gpio18.into_function::<FunctionSpi>();
        let mosi = pins.gpio19.into_function::<FunctionSpi>();
        let dc = pins.gpio20.into_function::<FunctionSioOutput>();
        let rst = pins.gpio21.into_function::<FunctionSioOutput>();
        let backlight = pins.gpio22.into_function::<FunctionSioOutput>();

        let spi_pin_layout = (mosi, sclk);
        let spi = Spi::<_, _, _, 8>::new(pac.SPI0, spi_pin_layout).init(
            &mut pac.RESETS,
            125u32.MHz(),
            64000u32.kHz(),
            MODE_0,
        );

        let spi_device =
            ExclusiveDevice::new(spi, cs, NoDelay).expect("failed to create Tft SPI device");
        tft = tft::Tft::new(spi_device, dc, rst, backlight, &mut timer);

        tft.backlight(true);
        tft.clear(Rgb565::WHITE);
        tft.println("Hello from RP2040", 100, 40);
    }

    {
        let cs = pins.gpio13.into_push_pull_output();
        let sclk: Pin<_, FunctionSpi, PullNone> = pins.gpio10.reconfigure();
        let mosi: Pin<_, FunctionSpi, PullNone> = pins.gpio11.reconfigure();
        let miso: Pin<_, FunctionSpi, PullUp> = pins.gpio12.reconfigure();

        let spi_pin_layout = (mosi, miso, sclk);

        let spi = Spi::<_, _, _, 8>::new(pac.SPI1, spi_pin_layout).init(
            &mut pac.RESETS,
            125u32.MHz(),
            400u32.kHz(),
            MODE_0,
        );

        let spi_device =
            ExclusiveDevice::new(spi, cs, NoDelay).expect("failed to create SD SPI dev");

        let mut sd = sd::SpiSD::new(spi_device, timer);
        // let mut y = 60;
        // sd.iterate_root_dir(|entry| {
        //     Tft.println(core::str::from_utf8(entry.name.base_name()).unwrap(), 40, y);
        //     y += 20;
        // })
        // .unwrap();

        let boot2_data =
            { unsafe { &*core::ptr::slice_from_raw_parts((0x1000_0000) as *const u8, 256_usize) } };
        info!("{}", &boot2_data);
        info!("{}", &BOOT2_FIRMWARE);

        let mut base_addr = 0;
        let mut flash_data: [u8; 4096] = [0; 4096];
        uf2::read_blocks(&mut sd, "ARCADE.UF2", |block| {
            if base_addr == 0 {
                base_addr = block.target_address;
            }
            if block.target_address < base_addr {
                core::panic!("target address went backwards");
            }
            let offset: usize = (block.target_address - base_addr)
                .try_into()
                .expect("target_address was > usize");
            let block_size: usize = block
                .payload_size
                .try_into()
                .expect("payload_size was > usize");

            info!(
                "block number {}, target 0x{:x}, size: {}, offset {}",
                block.block_number, block.target_address, block.payload_size, offset
            );

            if offset < 4096 {
                // Preserve our boot2, ignoring the one from the UF2.
                if block.target_address == 0x1000_0000 {
                    flash_data[offset..(offset + block_size)].copy_from_slice(&boot2_data[0..256]);
                    info!("{}", &block.data[0..256]);
                } else {
                    flash_data[offset..(offset + block_size)]
                        .copy_from_slice(&block.data[0..block_size]);
                }
            } else {
                info!("writing block at 0x{:x}", base_addr);
                if base_addr == 0x10000000 {
                    let mut x = 0;
                    while x < 4096 {
                        info!("{}", &flash_data[x..(x + 256)]);
                        x += 256;
                    }
                }
                flash::write_block(base_addr, &flash_data);

                flash_data[0..block_size].copy_from_slice(&block.data[0..block_size]);
                base_addr = block.target_address;
            }
        });
    }

    tft.println("Booting", 120, 60);
    boot::boot(XIP_BASE + 0x100);
}
