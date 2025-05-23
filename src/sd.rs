use defmt::*;
use embedded_hal::spi::MODE_0;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_sdmmc::{
    DirEntry, Error, File, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};
use fugit::RateExtU32;
use rp2040_hal::{
    gpio::{
        bank0::{Gpio10, Gpio11, Gpio12, Gpio13},
        FunctionSioOutput, FunctionSpi, Pin, PullDown, PullNone, PullUp,
    },
    pac::{self, SPI1},
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

pub type SdSpiDevice = ExclusiveDevice<
    Spi<
        Enabled,
        SPI1,
        (
            Pin<Gpio11, FunctionSpi, PullNone>,
            Pin<Gpio12, FunctionSpi, PullUp>,
            Pin<Gpio10, FunctionSpi, PullNone>,
        ),
    >,
    Pin<Gpio13, rp2040_hal::gpio::FunctionSio<rp2040_hal::gpio::SioOutput>, PullDown>,
    NoDelay,
>;

pub type SdFile<'a> = File<'a, SdCard<SdSpiDevice, Timer>, Clock, 4, 4, 1>;

pub struct SpiSD {
    volume_manager: VolumeManager<SdCard<SdSpiDevice, Timer>, Clock>,
}

impl SpiSD {
    pub fn new(
        resets: &mut rp2040_hal::pac::RESETS,
        spi: pac::SPI1,
        timer: rp2040_hal::Timer,
        mosi: Pin<Gpio11, FunctionSpi, PullNone>,
        miso: Pin<Gpio12, FunctionSpi, PullUp>,
        sclk: Pin<Gpio10, FunctionSpi, PullNone>,
        cs: Pin<Gpio13, FunctionSioOutput, PullDown>,
    ) -> SpiSD {
        let spi_pin_layout = (mosi, miso, sclk);

        let spi = Spi::<_, _, _, 8>::new(spi, spi_pin_layout).init(
            resets,
            125u32.MHz(),
            400u32.kHz(),
            MODE_0,
        );

        let spi_device =
            ExclusiveDevice::new(spi, cs, NoDelay).expect("failed to create SD SPI dev");

        let sdcard = SdCard::new(spi_device, timer);
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
