#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::console::putchar;

#[no_mangle]
fn main() -> i32 {
  putchar(b'H'); putchar(b'e'); putchar(b'l'); putchar(b'l'); putchar(b'o');
  0
}