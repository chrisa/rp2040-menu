use defmt::*;
use embassy_time::Delay;
use embedded_hal_02::spi::MODE_0;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{
    DirEntry, Error, File, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};

use embassy_rp::{
    gpio::{Level, Output},
    peripherals::SPI1,
    spi::{Async, Config as SpiConfig, Spi},
};

use crate::SdResources;

// This is just a placeholder TimeSource. In a real world application
// one would probably use the RTC to provide time.
pub struct Clock;

impl TimeSource for Clock {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

pub type SdSpiDevice<'spi> = ExclusiveDevice<Spi<'spi, SPI1, Async>, Output<'spi>, Delay>;

pub struct SpiSD<'spi> {
    volume_manager: VolumeManager<SdCard<SdSpiDevice<'spi>, Delay>, Clock>,
}

impl<'spi> SpiSD<'spi> {
    pub fn new(res: SdResources) -> SpiSD<'spi> {
        let mut spi_cfg = SpiConfig::default();
        spi_cfg.frequency = 12_000_000;
        spi_cfg.polarity = MODE_0.polarity;
        spi_cfg.phase = MODE_0.phase;

        let spi = Spi::new(
            res.spi, res.sclk, res.mosi, res.miso, res.tx_dma, res.rx_dma, spi_cfg,
        );

        let cs = Output::new(res.cs, Level::Low);
        let spi_delay = embassy_time::Delay;
        let spi_device = ExclusiveDevice::new(spi, cs, spi_delay)
            .expect("failed to create exclusive bus for sd");

        let timer = embassy_time::Delay;
        let sdcard = SdCard::new(spi_device, timer);
        info!(
            "Card size is {} bytes",
            sdcard.num_bytes().expect("failed to read size of card")
        );
        let timesource = Clock {};
        let volume_manager = VolumeManager::new(sdcard, timesource);
        SpiSD { volume_manager }
    }

    pub fn iterate_root_dir(
        &self,
        mut func: impl FnMut(&DirEntry),
    ) -> Result<(), Error<SdCardError>> {
        let volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        info!("Volume 0: {:?}", volume0);
        let root_dir = volume0.open_root_dir()?;
        root_dir
            .iterate_dir(|entry: &DirEntry| {
                info!("Entry: {}", defmt::Display2Format(&entry.name));
                func(entry);
            })
            .unwrap();
        Ok(())
    }

    pub fn open(
        &self,
        filename: &str,
        func: impl FnOnce(&File<'_, SdCard<SdSpiDevice<'_>, Delay>, Clock, 4, 4, 1>),
    ) -> Result<(), Error<SdCardError>> {
        let volume0 = self
            .volume_manager
            .open_volume(VolumeIdx(0))
            .expect("failed to open volume");
        let root_dir = volume0.open_root_dir().expect("failed to open root dir");
        let f = root_dir
            .open_file_in_dir(filename, embedded_sdmmc::Mode::ReadOnly)
            .expect("failed to open file");
        func(&f);
        Ok(())
    }
}
