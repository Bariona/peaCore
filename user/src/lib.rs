#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;

mod ds;
mod lang_items;
mod syscall;

use riscv::register::fcsr::Flags;
use syscall::*;

// ========= self-made allocator ==========
const USER_HEAP_SIZE: usize = 16384;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

use ds::buddy::LockedHeap;
#[global_allocator]
static HEAP: LockedHeap::<32> = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
  panic!("Heap allocation error, layout = {:?}", layout);
}
// ========================================

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
  unsafe {
    HEAP.lock()
      .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
  }
  exit(main());
  panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
  panic!("Cannot find main!");
}

use bitflags::bitflags;
bitflags! {
  pub struct OpenFlags: u32 {
    const RDONLY = 0;
    const WRONLY = 1 << 0;
    /// read & write
    const RDWR = 1 << 1;
    /// if the file doesn't exist, create it
    const CREATE = 1 << 9;
    /// clear file and return an empty one
    const TRUNC = 1 << 10;
  }
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
  sys_open(path, flags.bits)
}

pub fn close(fd: usize) -> isize {
  sys_close(fd)
}

pub fn read(fd: usize, buf: &[u8]) -> isize {
  sys_read(fd, buf)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
  sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
  sys_exit(exit_code)
}

pub fn fork() -> isize {
  sys_fork()
}

pub fn exec(path: &str) -> isize {
  sys_exec(path)
}

pub fn yield_() -> isize {
  sys_yield()
}
pub fn get_time() -> isize {
  sys_get_time()
}

pub fn getpid() -> isize {
  sys_getpid()
}

pub fn wait(exit_status: &mut i32) -> isize {
  loop {
    match sys_waitpid(-1, exit_status as *mut _) {
      -2 => {
        yield_();
      }
      exit_pid => {
        // -1 or a real_pid
        return exit_pid
      }
    }
  }
}

pub fn waitpid(pid: usize, exit_status: &mut i32) -> isize {
  loop {
    match sys_waitpid(pid as isize, exit_status as *mut _) {
      -2 => {
        yield_();
      }
      exit_pid => {
        // -1 or pid
        return exit_pid
      }
    }
  }
}

pub fn sleep(period_ms: usize) {
  let start = sys_get_time();
  loop {
    let cur = sys_get_time();
    if cur < start + period_ms as isize {
      sys_yield();
    } else {
      break;
    }
  }
}

pub fn sbrk(size: i32) -> isize {
  sys_sbrk(size)
}