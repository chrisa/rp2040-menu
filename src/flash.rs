use rp2040_flash::flash::*;

const FLASH_BASE: u32 = 0x1000_0000;

pub fn write_block(addr: u32, data: &[u8]) {
    unsafe {
        flash_range_erase_and_program(addr - FLASH_BASE, data, true);
    }
}
