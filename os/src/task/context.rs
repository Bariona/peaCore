use crate::trap::trap_return;

#[repr(C)]
pub struct TaskContext {
  ra: usize,
  /// **kernel stack** pointer
  sp: usize,
  /// callee saved registers
  /// 
  /// We don't need to store caller saved register, because during `switch()`, 
  /// the compiler will automatically store caller registers:
  /// 
  /// save caller regisers
  /// 
  /// call f()
  /// 
  /// restore caller
  s: [usize; 12],
}

impl TaskContext {
  /// init task context
  pub fn zero_init() -> Self {
    Self {
      ra: 0,
      sp: 0,
      s: [0; 12]
    }
  }

  /// set Task Context
  pub fn goto_trap_return(kernel_stack_ptr: usize) -> Self {
    Self {
      ra: trap_return as usize,
      sp: kernel_stack_ptr,
      s: [0; 12]
    }
  }
}