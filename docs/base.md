### QEMU

```
$ gdb --configuration 
```
can be used to check GDB's support architecture，`--target` parameter indicates the architecture it can debug.

If it doesn't support riscv64, try `riscv64-unknown-elf-gdb`.


- -s -S : open a remote localhost:1234
    ```
    $ target remote localhost:1234  # in gdb
    ```

### GDB
- b: create breakpoint
- cont: continue running
- n: run next command without entering the function
- ni: run next instruction...
- s: 
- si: 
- x/10i \<addr\>: shows the 10 instructions staring at addr
- wa: add watch point on var/mem


### Rust

- [#macro_use]: use the macro declared in the following crate.

- [#link_section]: 

    指定了输出对象文件中函数或静态项的内容将被放置到的节点位置。

    ```rust
    #[no_mangle]
    #[link_section = ".example_section"]
    pub static VAR1: u32 = 1;
    ```
- [build scripts: build.rs](https://course.rs/cargo/reference/build-script/intro.html)