use core::arch::global_asm;

use super::context::TaskContext;


global_asm!(include_str!("switch.S"));

extern "C" {
  /// Saving the current context in `current_task_cx_ptr`.
  /// Switch to the context of `next_task_cx_ptr`, 
  pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}