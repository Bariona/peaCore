use alloc::{sync::Arc};

use crate::{board::QEMUExit, fs::{open_file, Flags}};

use self::{task::{TaskControlBlock, TaskStatus}, context::TaskContext, processor::{take_current_task, schedule}};

mod context;
mod task_manager;
mod pid;
pub mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

pub use task_manager::add_task;

pub fn suspend_current_and_run_next() {
  let task = take_current_task().unwrap();
  
  // access PCB exclusively
  let mut task_inner = task.inner_exclusive_access();
  let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
  task_inner.task_status = TaskStatus::Ready;
  drop(task_inner);
  // release PCB

  // push_back to ready queue
  add_task(task);
  // goto scheduler
  schedule(task_cx_ptr);
}

/// pid of usertest
pub const IDLE_PID: usize = 0;

lazy_static! {
  pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
    println!("@@@@");
    let initproc = open_file("initproc", Flags::RDONLY).unwrap();
    TaskControlBlock::new(initproc.read_all().as_slice())
  });
}

pub fn exit_current_and_run_next(exit_code: i32) {
  let task = take_current_task().unwrap();

  let pid = task.getpid();
  if pid == IDLE_PID {
    println!(
      "[kernel] Idle process exit with exit_code {} ...",
      exit_code
    );
    // === exit kernel === 
    if exit_code != 0 {
      crate::board::QEMU_EXIT_HANDLE.exit_failure();
    } else {
      crate::board::QEMU_EXIT_HANDLE.exit_success();
    }
  }

  let mut task_inner = task.inner_exclusive_access();
  task_inner.task_status = TaskStatus::Zombie;
  task_inner.exit_code = exit_code;

  { // link zombie proc's childer to `initproc`
    let mut initproc_inner = INITPROC.inner_exclusive_access();
    for child in task_inner.children.iter() {
      child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
      initproc_inner.children.push(child.clone());
    }
  }

  task_inner.children.clear();
  task_inner.memory_set.recycle_data_pages();
  drop(task_inner);
  drop(task);

  let mut _unused = TaskContext::zero_init();
  schedule(&mut _unused as *mut TaskContext);
}

pub fn add_initproc() {
  add_task(INITPROC.clone());
}