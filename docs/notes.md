### How computer runs?

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


- Surpass your physical memory's constraint: 

    replacement policy [link](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/7more-as.html)


### RISCV

- pc：points to current instruction

    expect those instructions who modify pc (e.g. AUIPC), otherwise it will do `pc += 4` automatically.

- csrr/csrw: Control State Register Read/Write


- sepc: When a trap is taken into S-mode, sepc is written with the virtual address of the instruction that
encountered the exception.

- scause: When a trap is taken into S-mode, scause is written with a code indicating the event that caused the trap.

- sscratch: is used to hold a pointer to the hart-local supervisor context while the hart is
executing user code. 

    During the U-mode, sscratch points to TrapContext in peaCore.
> hart: hardware thread

- satp: This register holds the **physical** page number (PPN) of the root page table, i.e., its supervisor physical address
divided by 4 KiB.

> Form Rcore:
> 
> 而当 CPU 完成 Trap 处理准备返回的时候，需要通过一条 S 特权级的特权指令 sret 来完成，这一条指令具体完成以下功能：
>   
>   - CPU 会将当前的特权级按照 sstatus 的 SPP 字段设置为 U 或者 S ；
>
>   - CPU 会跳转到 sepc 寄存器指向的那条指令，然后继续执行。

When visited unallocated page: page fault.


### Makefile

- $@: 目标文件
