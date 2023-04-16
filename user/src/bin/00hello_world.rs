#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    let mut a = Vec::new();
    for i in 0..1000 {
        a.push(i);
    }
    0
}
