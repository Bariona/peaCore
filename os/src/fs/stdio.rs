use crate::{sbi::console_getchar, task::suspend_current_and_run_next};

use super::File;

pub struct Stdout;

pub struct Stdin;

impl File for Stdin {
  fn readable(&self) -> bool {
    true
  }

  fn writable(&self) -> bool {
    false
  }

  fn read(&self, mut buf: crate::mm::UserBuffer) -> usize {
    let mut c: usize;
    loop {
      c = console_getchar();
      assert_ne!(c, 0);
      if c == 0 {
        suspend_current_and_run_next();
        continue;
      } else {
        break;
      }
    }
    let ch = c as u8;
    assert_eq!(1, buf.len());
    unsafe {
      buf.buffers[0].as_mut_ptr().write_volatile(ch);
    }
    1
  }

  fn write(&self, _buf: crate::mm::UserBuffer) -> usize {
    panic!("cannot write to stdin");
  }
}

impl File for Stdout {
  fn readable(&self) -> bool {
    false
  }

  fn writable(&self) -> bool {
    true
  }

  fn read(&self, _buf: crate::mm::UserBuffer) -> usize {
    panic!("cannot read to stdout");
  }

  fn write(&self, buf: crate::mm::UserBuffer) -> usize {
    let mut len = 0;
    for byte in buf.buffers.iter() {
      print!("{}", core::str::from_utf8(byte).unwrap());
      len += byte.len();
    }
    len
  }
}