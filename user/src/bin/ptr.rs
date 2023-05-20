#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{console::putchar};

#[no_mangle]
fn main() -> i32 {
  let mut fnd: [u8; 4096] = [0; 4096];
  let mut fnd2: [u8; 4096] = [0; 4096];

  let mut stack: [u8; 2050] = [0; 2050];
  let mut stack2: [u8; 10] = [0; 10];

	for i in 0..2050 {
    fnd[i] = (i % 255) as u8;
  }
  stack[2049] = 1;
  for i in 0..2050 {
    stack[i] = fnd[i];
  }

  putchar(fnd[65]);

  for i in 0..2050 {
    fnd2[i] = (i % 255) as u8;
  }
  stack[2049] = 1;
  for i in 0..2050 {
    stack[i] = fnd2[i];
  }
  stack2[0] = fnd[66];
	stack2[1] = fnd[69];
  putchar(fnd2[65]);
  putchar(unsafe { *(&stack as *const [u8; 2050] as *const u8).offset(2050) });
  0
}