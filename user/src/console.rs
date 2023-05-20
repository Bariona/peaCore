use super::{read, write};

use core::fmt::{self, Write};

const STDIN: usize = 0;
const STDOUT: usize = 1;

struct Stdout;

impl Write for Stdout {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		write(STDOUT, s.as_bytes());
		Ok(())
	}
}

pub fn print(args: fmt::Arguments) {
	Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
	($fmt: literal $(, $($arg: tt)+)?) => {
		$crate::console::print(format_args!($fmt $(, $($arg)+)?));
	}
}

#[macro_export]
macro_rules! println {
	($fmt: literal $(, $($arg: tt)+)?) => {
		$crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
	}
}

pub fn putchar(c: u8) {
	// write(STDOUT, &[c]);
	print!("{}", c as char);
}

pub fn getchar() -> u8 {
	let c = [0u8; 1];
	read(STDIN, &c);
	c[0]
}

pub fn putint(x: isize) {
	print!("{}", x);
}

#[inline]
pub fn isdigit(ch: u8) -> bool {
	ch >= b'0' && ch <= b'9'
}

pub fn getint() -> isize {
	let mut c = getchar();
	let mut ret: isize = 0;
	let mut f: isize = 1;

	while !isdigit(c) {
		// println!("{}# read", c as char);
		if c == b'-' {
			f = -1;
		}
		c = getchar();
	}

  while isdigit(c) {
    ret = ret * 10 + (c as isize - b'0' as isize);
    c = getchar();
	}
	ret * f
}