use defmt::*;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_sdmmc::{
    DirEntry, Error, File, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};
use rp2040_hal::{
    gpio::{
        bank0::{Gpio10, Gpio11, Gpio12, Gpio13},
        FunctionSioOutput, FunctionSpi, Pin, PullDown, PullNone, PullUp,
    },
    pac::SPI1,
    spi::{Enabled, Spi},
    Timer,
};

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

type SdSpi<S, Tx, Rx, Sck> = Spi<
    Enabled,
    S,
    (
        Pin<Tx, FunctionSpi, PullNone>,
        Pin<Rx, FunctionSpi, PullUp>,
        Pin<Sck, FunctionSpi, PullNone>,
    ),
>;

type SdSpiDeviceAny<S, Tx, Rx, Sck, Cs> = ExclusiveDevice<SdSpi<S, Tx, Rx, Sck>, Cs, NoDelay>;

type SdSpiDevice1 =
    SdSpiDeviceAny<SPI1, Gpio11, Gpio12, Gpio10, Pin<Gpio13, FunctionSioOutput, PullDown>>;

type SdSpiDevice = SdSpiDevice1;

pub type SdFile<'a> = File<'a, SdCard<SdSpiDevice, Timer>, Clock, 4, 4, 1>;

pub struct SpiSD {
    volume_manager: VolumeManager<SdCard<SdSpiDevice, Timer>, Clock, 4, 4, 1>,
}

impl SpiSD {
    pub fn new(spi_device: SdSpiDevice, delay: Timer) -> SpiSD {
        let sdcard = SdCard::new(spi_device, delay);
        info!(
            "Card size is {} bytes",
            sdcard.num_bytes().expect("failed to read size of card")
        );
        let timesource = Clock {};
        let volume_manager = VolumeManager::new(sdcard, timesource);
        SpiSD { volume_manager }
    }

    pub fn iterate_root_dir<F>(&mut self, mut func: F) -> Result<(), Error<SdCardError>>
    where
        F: FnMut(&DirEntry),
    {
        let mut volume0 = self.volume_manager.open_volume(VolumeIdx(0))?;
        info!("Volume 0: {:?}", volume0);
        let mut root_dir = volume0.open_root_dir()?;
        root_dir
            .iterate_dir(|entry: &DirEntry| {
                info!("Entry: {}", defmt::Display2Format(&entry.name));
                func(entry);
            })
            .unwrap();
        Ok(())
    }

    pub fn open<F>(&mut self, filename: &str, mut func: F) -> Result<(), Error<SdCardError>>
    where
        F: FnMut(&mut SdFile<'_>),
    {
        let mut volume0 = self
            .volume_manager
            .open_volume(VolumeIdx(0))
            .expect("failed to open volume");
        let mut root_dir = volume0.open_root_dir().expect("failed to open root dir");
        let mut f = root_dir
            .open_file_in_dir(filename, embedded_sdmmc::Mode::ReadOnly)
            .expect("failed to open file");
        func(&mut f);
        Ok(())
    }
}
