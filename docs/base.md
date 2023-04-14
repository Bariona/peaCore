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

- ctrl+A C: QEMU monitor 

    ```
    info mem
    info registers
    ```

    ​

### GDB

- b: create breakpoint

- bt: display the call stack(backtrace) of the program

   For example, `bt 5` will only display the first 5 levels of the call stack of the current execution context.

- cont: continue running

- n: run next command without entering the function

- ni: run next instruction...

- s: 

- si: 

- x/10i \<addr\>: shows the 10 instructions staring at addr

- p/x $pc: show register's value

- wa: add watch point on var/mem

- undisplay 3: del 3rd breakpint