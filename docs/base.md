### QEMU

```
$ gdb --configuration 
```
can be used to check GDB's support architecture，`--target` parameter indicates the architecture it can debug.

If it doesn't support riscv64, try `riscv64-unknown-elf-gdb`.


- -s -S : open a remote localhost:1234
    ```
    $ target remote localhost:1234
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