#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

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
mod sync;
mod sbi;
mod timer;
mod syscall;
mod lang_items; // panic_handler
mod vga_buffer;
mod mm;
mod task;
mod trap;
mod loader;

use core::arch::global_asm;

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
	// print_something();
	clear_bss();
	println!("[kernel] Hello, OS World!");
	ds::test();
	mm::init();
	mm::heap_test();
	mm::remap_test();
	trap::init();
	trap::enable_timer_interrupt();
	timer::set_next_trigger();
	task::run_first_task();
	panic!("Unreachable in kernel");
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