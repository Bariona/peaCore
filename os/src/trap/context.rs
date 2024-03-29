//! Implement [`TrapContext`]

use riscv::register::sstatus::{Sstatus, self, SPP};

#[repr(C)]
pub struct TrapContext {
  /// general regs[0..31]
  pub x: [usize; 32],
  pub sstatus: Sstatus,
  pub sepc: usize,

  /// addr of kernel's page table
  pub kernel_satp: usize,
  /// kernel stack pointer
  pub kernel_sp: usize,
  /// addr of trap_handler 
  pub trap_handler: usize
}

impl TrapContext {
  /// x2 (ABI Name = sp)
  pub fn set_sp(&mut self, sp: usize) {
    self.x[2] = sp;
  }

  /// init 
  pub fn app_init_context(
    entry: usize,
    sp: usize,
    kernel_satp: usize,
    kernel_sp: usize, 
    trap_handler: usize
  ) -> Self {
    let mut sstatus = sstatus::read();
    sstatus.set_spp(SPP::User);
    let mut cx = Self {
      x: [0; 32],
      sstatus, 
      sepc: entry,  // entry point of app
      kernel_satp,  
      kernel_sp,
      trap_handler
    };
    cx.set_sp(sp); // app's user stack pointer
    cx
  }
}

