#![no_std]
#![no_main]

use assign_resources::assign_resources;
use embassy_executor::Spawner;
use embassy_rp::peripherals;
use embassy_rp::Peri;
use static_cell::StaticCell;

use defmt_rtt as _;
use panic_probe as _;

use config::CONFIG_ILI9341;
use rp2040_boot2::BOOT_LOADER_W25Q080_TOP64K;

use crate::sd::SpiSD;
use crate::tft::Tft;

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

assign_resources! {
    tft: TftResources {
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

static TFT: StaticCell<Tft<'_>> = StaticCell::new();
static SD: StaticCell<SpiSD<'_>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    let tft: &mut Tft<'_> = TFT.init(tft::Tft::new(r.tft).await);
    tft.backlight(true).await;
    let mut ui = ui::UI::new(tft);
    ui.init().await;

    let sd: &'static SpiSD<'_> = SD.init(sd::SpiSD::new(r.sd));
    let mut fw = flash::FlashWriter::new();
    uf2::read_blocks(&sd, "ARCADE.UF2", |block| {
        fw.next_block(block);
    });

    boot::boot(XIP_BASE + 0x100);
}
