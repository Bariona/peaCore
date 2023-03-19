#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
mod console;

#[macro_use]
extern crate lazy_static;

mod config;
mod sync;
mod sbi;
mod lang_items; // panic_handler
mod mm;

use core::arch::global_asm;

use crate::sbi::shutdown;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
	clear_bss();
	println!("Hello, World!");
	shutdown();
}

fn clear_bss() {
	extern "C" {
		fn sbss();
		fn ebss();
	}
	(sbss as usize..ebss as usize).for_each(|a| {
		println!("write {}", a);
		unsafe { (a as *mut u8).write_volatile(0) }
	});
}