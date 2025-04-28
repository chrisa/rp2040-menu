MEMORY {
    BOOT2  : ORIGIN = 0x10000000, LENGTH = 0x100
    CONFIG : ORIGIN = 0x100FF000, LENGTH = 0x100
    FLASH  : ORIGIN = 0x10000000 + 2M - 128K, LENGTH = 64K
    RAM    : ORIGIN = 0x20000000, LENGTH = 256K
}

EXTERN(BOOT2_FIRMWARE)
EXTERN(CONFIG)

SECTIONS {
    /* ### Boot loader */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2

    /* ### Config */
    .config ORIGIN(CONFIG) :
    {
        KEEP(*(.config));
    } > CONFIG
} INSERT BEFORE .text;
