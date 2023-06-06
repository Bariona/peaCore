use alloc::sync::Arc;

use crate::{task::{exit_current_and_run_next, suspend_current_and_run_next, processor::{current_user_token, current_task}, add_task}, timer::get_time_ms, mm::{translated_str, translated_refmut}, fs::{open_file, Flags}};

/// exit current task
pub fn sys_exit(exit_code: i32) -> ! {
  println!("[kernel] Application exited with code {}", exit_code);
  exit_current_and_run_next(exit_code);
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


pub fn sys_fork() -> isize {
  let current_task = current_task().unwrap();
  let new_task = current_task.fork();
  let new_pid = new_task.pid.0;

  let new_trap_cx = new_task.inner_exclusive_access().get_trap_cx();
  new_trap_cx.x[10] = 0;

  add_task(new_task);
  // println!("[kernel] fork: {} {}", current_task.getpid(), new_pid);
  new_pid as isize
}

pub fn sys_exec(path_ptr: *const u8) -> isize {
  let token = current_user_token();
  let path = translated_str(token, path_ptr);
  // println!("{} {}", current_task().unwrap().pid.0, path);

  if let Some(file) = open_file(path.as_str(), Flags::RDONLY) {
    // println!("{} {}", current_task().unwrap().pid.0, path);
    let task = current_task().unwrap();
    task.exec(file.read_all().as_slice());
    0
  } else {
    -1
  }
}

pub fn sys_getpid() -> isize {
  current_task().unwrap().pid.0 as isize
}

/// Return -1 if no child proc (pid = -1) or no corresponding child proc (pid != -1)
/// Return -2 if the candidate proc is still not `Zombie`
pub fn sys_waitpid(pid: isize, exit_status: *mut i32) -> isize {
  let task = current_task().unwrap();

  // println!("[kernel] waiter pid {} waitee pid {}", task.getpid(), pid);
  let mut inner = task.inner_exclusive_access();
  if !inner
    .children
    .iter()
    .any(|child| pid == -1 || child.getpid() == pid as usize) {
      return -1;
  }
  let pair = inner
    .children
    .iter()
    .enumerate()
    .find(|(_, p)| {
      p.inner_exclusive_access().is_zombie() && (pid == -1 || p.getpid() == pid as usize)
  });

  if let Some((idx, _)) = pair {
    let child = inner.children.remove(idx);
    assert_eq!(Arc::strong_count(&child), 1);
    let found_pid = child.getpid();
    let exit_code = child.inner_exclusive_access().exit_code;
    *translated_refmut(inner.get_user_token(), exit_status) = exit_code;
    found_pid as isize 
  } else {
    -2 // not a zombie proc
  }
}

// /// change `data` segmemnt size
// pub fn sys_sbrk(size: i32) -> isize {
//   if let Some(old_brk) = change_program_brk(size) {
//     old_brk as isize
//   } else {
//     -1
//   }
// }