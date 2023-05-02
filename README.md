## <img src="docs/static/icon.png" width="55"> peaCore

![](https://img.shields.io/badge/language-Rust-brightgreen)

peaCore is a toy kernel implemented by Rust language.

### Support features
 to do

### Prequisite

#### GDB: 

gdb-multiarch (support riscv64 with `TUI` mode)

```shell
sudo apt-get install gdb-multiarch
```

#### QEMU

`-machine virt`: 1 NS16550 compatible UART

### To Do list

- [x] remove ecall (rustsbi)
  
    1. early-output[UART0: 0x1000_0000] 

    2. shutdown (sys_exit)

    3. `cargo test` for robustness

      > meet error... (to do)



- [x] Self-made Buddy for heap allocation

  Ref: [rust-sbi CHANGELOG](https://github.com/rustsbi/rustsbi/blob/91cfa36d14b81af3874ba1da2c0663b5bd601fa3/CHANGELOG.md?plain=1#L122), rust-sbi-tutorial

- [ ] Slab

- [x] Timer interrupt: remove from rust-sbi

- [ ] Trap SIP clear when trap_return

- [ ] sbrk

- [ ] Random location memory

- [ ] Process scheduler


### Reference

- [rCore-Tutorial](https://rcore-os.cn/rCore-Tutorial-Book-v3/index.html)

- [xv6](https://github.com/mit-pdos/xv6-riscv)

- [Blog OS](https://os.phil-opp.com/)

- [rust-lang](https://doc.rust-lang.org/std/index.html)
