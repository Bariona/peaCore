#![no_std]
#![no_main]

extern crate user_lib;

use core::ptr;

use user_lib::{console::{putchar, getint, putint}, println};

#[no_mangle]
fn main() -> i32 {
  // loop {
  //   let x = getint();
  //   putint(x);
  //   println!("");
  //   // putchar(b'#');
  // }
  let p: *mut i32 = ptr::null_mut();
  unsafe {
    *p = 10;
  }
  putchar(b'F');
  putchar(b'a');
  putchar(b'i');
  putchar(b'l');
  loop {}
  0
}