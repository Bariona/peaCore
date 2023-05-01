#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(fn_align)]

/*
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
*/

extern crate alloc;

#[macro_use]
mod console;

#[macro_use]
extern crate lazy_static;

#[path = "boards/qemu.rs"]
mod board;

mod config;
mod ds;
mod start;
mod sync;
mod sbi;
mod syscall;
mod timer;
mod task;
mod trap;
mod loader;
mod lang_items; // panic_handler
mod vga_buffer;
mod mm;
mod uart;

use core::{arch::global_asm};

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/*
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
	for test in tests {
		test();
	}
}
*/

#[no_mangle]
pub fn rust_main() -> ! {
	println!("[kernel] Hello, OS World!");
	ds::test();
	mm::init();
	mm::remap_test();
	trap::init();

	task::add_initproc();
	loader::list_apps();
	// trap::enable_timer_interrupt();
	// timer::set_next_trigger();
	task::processor::run_tasks();
	panic!("Unreachable in kernel");
}