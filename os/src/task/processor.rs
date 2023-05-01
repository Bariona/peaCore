//!Implementation of [`Processor`] and Intersection of control flow

use alloc::sync::Arc;

use crate::{sync::up::UPSafeCell, trap::context::TrapContext};

use super::{task::{TaskControlBlock, TaskStatus}, context::TaskContext, task_manager::fetch_task, switch::__switch};
 

/// Processor management structure
pub struct Processor {
  ///The task currently executing on the current processor
  current: Option<Arc<TaskControlBlock>>,
  ///The basic control flow of each core, helping to select and switch process
  idle_task_cx: TaskContext
}

impl Processor {
  pub fn new() -> Self {
    Self { 
      current: None, 
      idle_task_cx: TaskContext::zero_init()
    }
  }

  /// Return a mut ref to `idle_task_cx` 
  fn get_idle_task_cx(&mut self) -> *mut TaskContext {
    &mut self.idle_task_cx as *mut _
  }

  /// Return current task in moving semanteme
  /// Takes the value out of the option, leaving a None in its place
  pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
    self.current.take()
  }

  /// Return current task in cloning semanteme 
  pub fn current(&mut self) -> Option<Arc<TaskControlBlock>>{
    self.current.as_ref().map(Arc::clone)
  }
}

lazy_static! {
  pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe {
    UPSafeCell::new(Processor::new())
  };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
  loop {
    let mut processor = PROCESSOR.exclusive_access();
    if let Some(task) = fetch_task() {
      let idle_task_cx_ptr = processor.get_idle_task_cx();
      let mut task_inner = task.inner_exclusive_access();
      let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
      task_inner.task_status = TaskStatus::Running;
      drop(task_inner); // release coming task TCB manually
      processor.current = Some(task);
      drop(processor); // release processor manually
      unsafe {
        __switch(idle_task_cx_ptr, next_task_cx_ptr)
      }
    }
  }
}

///Take the current task,leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
  PROCESSOR.exclusive_access().take_current()
}

///Get running task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
  PROCESSOR.exclusive_access().current()
}

///Get token of the address space of current task
pub fn current_user_token() -> usize {
  let task = current_task().unwrap();
  let token = task.inner_exclusive_access().get_user_token();
  token
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
  current_task().unwrap().inner_exclusive_access().get_trap_cx()
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
  let mut processor = PROCESSOR.exclusive_access();
  let idle_task_cx_ptr = processor.get_idle_task_cx();
  drop(processor); // must drop processor manually before __switch
  unsafe {
    __switch(switched_task_cx_ptr, idle_task_cx_ptr)
  }
}