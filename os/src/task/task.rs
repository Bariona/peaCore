use crate::{mm::{memory_set::{MemorySet, KERNEL_SPACE, MapPermission}, address::{PhysPageNum, VirtAddr}}, config::TRAP_CONTEXT, trap::{kernel_stack_position, context::TrapContext, trap_handler}};

use super::context::TaskContext;

#[derive(PartialEq, Clone, Copy)]
pub enum TaskStatus {
  Ready,
  Running,
  Exited
}

pub struct TaskControlBlock {
  pub task_status: TaskStatus,
  pub task_cx: TaskContext,
  pub memory_set: MemorySet,    /// task's user memory space
  pub trap_cx_ppn: PhysPageNum, /// trap function() (i.e. trampoline)'s physical page number
  pub base_size: usize,
  pub heap_bottom: usize,
  pub program_brk: usize,
}

impl TaskControlBlock {
  pub fn get_trap_cx(&self) -> &'static mut TrapContext {
    self.trap_cx_ppn.get_mut()
  }

  pub fn get_user_token(&self) -> usize {
    self.memory_set.token()
  }

  pub fn new(elf_data: &[u8], app_id: usize) -> Self {
    // println!("id = {}", app_id);
    let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    // note that `memory_set` is for the user address space
    // but, ... we are currently in the S-mode
    let trap_cx_ppn = memory_set
      .translate(VirtAddr::from(TRAP_CONTEXT).into())
      .unwrap()
      .ppn();
    // allocate process's kernel stack
    let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
    KERNEL_SPACE.exclusive_access().insert_framed_area(
      kernel_stack_bottom.into(), 
      kernel_stack_top.into(), 
      MapPermission::R | MapPermission::W
    );
    let task_control_block = Self {
      task_status: TaskStatus::Ready,
      // set the trapContext at the top of the stack
      task_cx: TaskContext::goto_trap_return(kernel_stack_top),
      memory_set,
      trap_cx_ppn,
      base_size: user_sp,
      heap_bottom: user_sp,
      program_brk: user_sp
    };
    let trap_cx = task_control_block.get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
      entry_point, 
      user_sp, 
      KERNEL_SPACE.exclusive_access().token(), 
      kernel_stack_top, 
      trap_handler as usize
    );
    task_control_block
  }

  /// change the location of the program break. 
  /// 
  /// return None if failed, else return Some(old_brk)
  pub fn change_program_brk(&mut self, size: i32) -> Option<usize> {
    let old_brk = self.program_brk;
    let new_brk = (self.program_brk as isize + size as isize) as usize;
    if (new_brk as isize) < (self.heap_bottom as isize) {
      return None
    } 
    let result = if size < 0 {
      self.memory_set.shrink_to(
        self.heap_bottom.into(), // TODO: VirtAddr(new_brk) ?
        new_brk.into()
      )
    } else {
      self.memory_set.append_to(
        self.heap_bottom.into(),
        new_brk.into()
      )
    };
    if result {
      self.program_brk = new_brk;
      Some(old_brk)
    } else {
      None
    }
  }
}