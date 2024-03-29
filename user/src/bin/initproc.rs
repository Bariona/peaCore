#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{fork, exec, wait, yield_};

#[no_mangle]
fn main() -> i32 {  
  if fork() == 0 {
    exec("user_shell\0");
  } else {
    loop {
      let mut exit_code: i32 = 0;
      let pid = wait(&mut exit_code);
      if pid == -1 {
        yield_(); // no proc is now a zombie
      } else {
        println!(
          "[initproc] Released a zombie process, pid = {}, exit_code = {}",
          pid, exit_code,
        );
      }
    }
  }
  0
}
