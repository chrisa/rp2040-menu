use defmt::info;
use uf2_block::Block;

const UF2_BLOCK_LENGTH: usize = 512;

use crate::sd::SpiSD;

pub fn read_blocks(sd: &mut SpiSD, filename: &str) {
    sd.open(filename, |file| {
        while !file.is_eof() {
            let mut buf: [u8; UF2_BLOCK_LENGTH] = [0; UF2_BLOCK_LENGTH];
            file.read(&mut buf).expect("failed to read");
            let block = Block::parse(&buf).expect("failed to parse");
            info!("{}", defmt::Display2Format(&block));
        }
    })
    .expect("failed to open");
}
