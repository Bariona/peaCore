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
mod vga_buffer;
mod mm;

use core::arch::global_asm;

use crate::sbi::shutdown;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
	// print_something();
	clear_bss();
	println!("Hello, OS World!");
	mm::init();
	shutdown();
}

fn clear_bss() {
	extern "C" {
		fn sbss();
		fn ebss();
	}
	unsafe {
		core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
			.fill(0);
	}
}