### How PC runs?

[ref](https://www.ruanyifeng.com/blog/2013/02/booting.html)

1. power on/off: CPU resets status

    - e.g. EIP($pc in x86) = 0x0000fff0

    This usually means a jmp instruction to the `firmware` in the **ROM**.

2. Firmware: BIOS / UEFI

    - Legacy BIOS (Basic I/O System)

    - UEFI (Unified Extensible Firmware Interface)

    Then, the firmware takes place to **load data into memory**.

    Legacy BIOS takes the first sector(扇区, roughtly 512B) into 0x7c00 in memory. 

    > the first sector should end with 0xaa55 and it's called MBR.

3. MBR (Master boot record 主引导记录)

    usually it tells the computer how to find the kernel code.