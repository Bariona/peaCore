use alloc::{sync::Arc};

use crate::{loader::get_app_data_by_name, board::QEMUExit};

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
  pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
    TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
  );
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
/*
pub struct TaskManager {
  num_app: usize,
  inner: UPSafeCell<TaskManagerInner>
}

struct TaskManagerInner {
  tasks: Vec<TaskControlBlock>,
  current_task: usize
}

lazy_static! {
  pub static ref TASK_MANAGER: TaskManager = {
    println!("\x1b[92minit TASK MANAGER\x1b[0m");
    let num_app = get_num_app();
    println!("\x1b[93mnum_app = {}\x1b[0m", num_app);
    let mut tasks = Vec::new();
    for i in 0..num_app {
      tasks.push(TaskControlBlock::new(get_app_data(i), i));
    }
    TaskManager {
      num_app, 
      inner: unsafe {
        UPSafeCell::new(TaskManagerInner{ 
          tasks,
          current_task: 0
        })
      }
    }
  };
}

impl TaskManager {
  fn run_first_task(&self) -> ! {
    let mut inner = self.inner.exclusive_access();
    let next_task = &mut inner.tasks[0];
    next_task.task_status = TaskStatus::Running;
    let next_task_cx_ptr = &next_task.task_cx as *const TaskContext;
    drop(inner);
    let mut _unused = TaskContext::zero_init();
    unsafe {
      __switch(&mut _unused as *mut _, next_task_cx_ptr);
    }
    panic!("unreachable in run_first_task!");
  }

  /// current process: Running -> Ready
  fn mark_current_suspended(&self) {
    let mut inner = self.inner.exclusive_access();
    let cur = inner.current_task;
    inner.tasks[cur].task_status = TaskStatus::Ready;
  }

  /// current process: Running -> Exited
  fn mark_current_exited(&self) {
    let mut inner = self.inner.exclusive_access();
    let cur = inner.current_task;
    inner.tasks[cur].task_status = TaskStatus::Exited;
  }

  /// find test task whose status = Ready, otherwise returns `None`
  fn find_next_task(&self) -> Option<usize> {
    let inner = self.inner.exclusive_access();
    let cur = inner.current_task;
    (cur + 1..cur + 1 + self.num_app)
      .map(|id| id % self.num_app)
      .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
  }

  #[allow(unused)]
  fn get_current_task_id(&self) -> usize {
    let inner = self.inner.exclusive_access();
    inner.current_task
  }

  fn get_current_token(&self) -> usize {
    let inner = self.inner.exclusive_access();
    inner.tasks[inner.current_task].get_user_token()
  }

  /// get trapContext in virtual addr: [trampoline - PAGE_SIZE, trampoline)
  fn get_current_trap_cx(&self) -> &'static mut TrapContext {
    let inner = self.inner.exclusive_access();
    inner.tasks[inner.current_task].get_trap_cx()
  }

  fn change_current_program_brk(&self, size: i32) -> Option<usize> {
    let mut inner = self.inner.exclusive_access();
    let cur = inner.current_task;
    inner.tasks[cur].change_program_brk(size)
  }

  fn run_next_app(&self) {
    if let Some(next) = self.find_next_task() {
      let mut inner = self.inner.exclusive_access();
      let cur = inner.current_task;
      inner.tasks[next].task_status = TaskStatus::Running;
      inner.current_task = next;
      let current_task_cx_ptr = &mut inner.tasks[cur].task_cx as *mut TaskContext;
      let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
      drop(inner);
      unsafe { __switch(current_task_cx_ptr, next_task_cx_ptr); }
    } else {
      println!("All applications completed!");
      use crate::board::QEMUExit;
      crate::board::QEMU_EXIT_HANDLE.exit_success();
    }
  }
}

/// Run the first task in task list.
pub fn run_first_task() {
  TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
  TASK_MANAGER.run_next_app();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
  TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
  TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
  mark_current_suspended();
  run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
  mark_current_exited();
  run_next_task();
}

/// Get the current task's id
#[allow(unused)]
pub fn current_task_id() -> usize {
  TASK_MANAGER.get_current_task_id()
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
  TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
  TASK_MANAGER.get_current_trap_cx()
}

/// Change the current 'Running' task's program break
pub fn change_program_brk(size: i32) -> Option<usize> {
  TASK_MANAGER.change_current_program_brk(size)
}
*/