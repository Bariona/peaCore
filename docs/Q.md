- [ ] MapArea 为什么要用BTree

- [ ]  内核既然是identical mapping 为什么需要启动页表?

    以及identical mapping有什么好处?

    TrapContext放置在什么位置里? (虚拟内存地址放在Trapoline之后的4k中)

- [x]  MMIO 忘记mapping, 会导致一些奇怪的错误, 比如退出QEMU的时候, stvec被写成了U-mode的trap_handler,
   但理论上应该是kernel的trap_handler, 修改了以后就对了, 不知道是什么问题...
    
- [x] Q: 在产生trap前后的一小段时间内会有一个比较**极端**的情况，即刚产生trap时，CPU已经进入了Supervisor Mode，但此时执行代码和访问数据还是在应用程序所处的用户态虚拟地址空间中，而不是我们通常理解的内核虚拟地址空间。在这段特殊的时间内，CPU指令为什么能够被连续执行呢？

    A: 这里需要注意：无论是内核还是应用的地址空间，跳板的虚拟页均位于同样位置，且它们也将会映射到同一个实际存放这段汇编代码的物理页帧。也就是说，在执行 __alltraps 或 __restore 函数进行地址空间切换的时候，应用的用户态虚拟地址空间和操作系统内核的内核态虚拟地址空间对切换地址空间的指令所在页的映射方式均是相同的，这就说明了这段切换地址空间的指令控制流仍是可以连续执行的。


- [x] Exception 和 interrupt, trap的区别到底是什么?
- [x] U-mode 中的stack由buddy接管, 但是既然大小不能改变, brk()改变的又是什么?
    用户的堆空间