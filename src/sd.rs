use defmt::*;
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use embedded_sdmmc::{
    DirEntry, Error, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};
use rp2040_hal::{
    gpio::{FunctionSpi, Pin, PinId, PullNone, PullUp},
    spi::{Enabled, Spi, SpiDevice, ValidPinIdRx, ValidPinIdSck, ValidPinIdTx},
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

type SdSpiDevice<S, Tx, Rx, Sck, Cs> = ExclusiveDevice<SdSpi<S, Tx, Rx, Sck>, Cs, NoDelay>;

pub struct SpiSD<S, Tx, Rx, Sck, Cs>
where
    S: SpiDevice,
    Tx: PinId + ValidPinIdTx<S>,
    Rx: PinId + ValidPinIdRx<S>,
    Sck: PinId + ValidPinIdSck<S>,
    Cs: OutputPin,
{
    sdcard: SdCard<SdSpiDevice<S, Tx, Rx, Sck, Cs>, Timer>,
    timesource: Clock,
}

impl<S, Tx, Rx, Sck, Cs> SpiSD<S, Tx, Rx, Sck, Cs>
where
    S: SpiDevice,
    Tx: PinId + ValidPinIdTx<S>,
    Rx: PinId + ValidPinIdRx<S>,
    Sck: PinId + ValidPinIdSck<S>,
    Cs: OutputPin,
{
    pub fn new(
        spi_device: SdSpiDevice<S, Tx, Rx, Sck, Cs>,
        delay: Timer,
    ) -> SpiSD<S, Tx, Rx, Sck, Cs> {
        let sdcard = SdCard::new(spi_device, delay);
        SpiSD {
            sdcard,
            timesource: Clock {},
        }
    }

    pub fn iterate_root_dir<F>(self, mut func: F) -> Result<(), Error<SdCardError>>
    where
        F: FnMut(&DirEntry),
    {
        info!("Card size is {} bytes", self.sdcard.num_bytes()?);
        let mut volume_mgr = VolumeManager::new(self.sdcard, self.timesource);
        let mut volume0 = volume_mgr.open_volume(VolumeIdx(0))?;
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
}
