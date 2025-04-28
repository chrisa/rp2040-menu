use uf2_block::Block;

const UF2_BLOCK_LENGTH: usize = 512;

use crate::sd::SpiSD;

pub fn read_blocks<F>(sd: &mut SpiSD, filename: &str, mut func: F)
where
    F: FnMut(&Block),
{
    sd.open(filename, |file| {
        while !file.is_eof() {
            let mut buf: [u8; UF2_BLOCK_LENGTH] = [0; UF2_BLOCK_LENGTH];
            file.read(&mut buf).expect("failed to read");
            let block = Block::parse(&buf).expect("failed to parse");
            func(&block);
        }
    })
    .expect("failed to open");
}
