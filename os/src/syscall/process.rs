use crate::{task::{exit_current_and_run_next, suspend_current_and_run_next, change_program_brk}, timer::get_time_ms};

/// exit current task
pub fn sys_exit(exit_code: i32) -> ! {
  println!("[kernel] Application exited with code {}", exit_code);
  exit_current_and_run_next();
  panic!("Unreachable in sys_exit!")
}

/// give up current running task and yield
pub fn sys_yield() -> isize {
  suspend_current_and_run_next();
  0
}

/// get current time
pub fn sys_get_time() -> isize {
  get_time_ms() as isize
}

/// change `data` segment size
pub fn sys_sbrk(size: i32) -> isize {
  if let Some(old_brk) = change_program_brk(size) {
    old_brk as isize
  } else {
    -1
  }
}