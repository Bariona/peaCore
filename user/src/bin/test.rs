#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exec, fork, getpid, wait};

#[no_mangle]
pub fn main() -> i32 {
  println!("test begin:");
  let a = exec("hello_world\0");
  println!("a = {}, test end", a);
  0
}
