use core::arch::asm;

const PPB_BASE: u32 = 0xe0000000;
const M0PLUS_VTOR_OFFSET: u32 = 0x0000ed08;

pub fn boot(dest: u32) -> ! {
    unsafe {
        asm!(
            ".thumb",
            ".syntax unified",
            ".cpu cortex-m0plus",
            "str r0, [r1]",
            "ldmia r0, {{r0, r1}}",
            "msr msp, r0",
            "bx r1",
            in("r0") dest,
            in("r1") PPB_BASE + M0PLUS_VTOR_OFFSET,
            options(noreturn)
        );
    }
}
