#![no_std]
#![no_main]

extern crate user_lib;

use core::arch::asm;

use user_lib::{console::putchar};

#[no_mangle]
fn main() -> i32 {
  unsafe{
    asm!(".byte 0x00, 0x00, 0x00");
  }
  putchar(b'F');
  putchar(b'a');
  putchar(b'i');
  putchar(b'l');
  loop {}
  0
}