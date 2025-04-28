use rp2040_flash::flash::*;
use uf2_block::Block;

use crate::BOOT2_FIRMWARE;

const FLASH_BASE: u32 = 0x1000_0000;
const FLASH_BLOCK_SIZE: usize = 4096;

pub struct FlashWriter {
    base_addr: u32,
    flash_data: [u8; FLASH_BLOCK_SIZE],
}

impl FlashWriter {
    pub fn new() -> Self {
        Self {
            base_addr: 0,
            flash_data: [0; FLASH_BLOCK_SIZE],
        }
    }

    pub fn next_block(&mut self, block: &Block) {
        if self.base_addr == 0 {
            self.base_addr = block.target_address;
        }
        if block.target_address < self.base_addr {
            core::panic!("target address went backwards");
        }
        let offset: usize = (block.target_address - self.base_addr)
            .try_into()
            .expect("target_address was > usize");
        let block_size: usize = block
            .payload_size
            .try_into()
            .expect("payload_size was > usize");

        if offset < 4096 {
            // Preserve our boot2, ignoring the one from the UF2.
            if block.target_address == 0x1000_0000 {
                self.flash_data[offset..(offset + block_size)].copy_from_slice(&BOOT2_FIRMWARE);
            } else {
                self.flash_data[offset..(offset + block_size)]
                    .copy_from_slice(&block.data[0..block_size]);
            }
        } else {
            self.write_block();

            self.flash_data[0..block_size].copy_from_slice(&block.data[0..block_size]);
            self.base_addr = block.target_address;
        }
    }

    fn write_block(&self) {
        unsafe {
            flash_range_erase_and_program(self.base_addr - FLASH_BASE, &self.flash_data, true);
        }
    }
}
