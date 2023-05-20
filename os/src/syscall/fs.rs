//! File and filesystem-related syscalls

use crate::{mm::translated_byte_buffer, task::processor::current_user_token, sbi::console_getchar};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// write buf of length `len` to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
  match fd {
    FD_STDOUT => {
      let buffers = translated_byte_buffer(current_user_token(), buf, len);
      for byte in buffers {
        print!("{}", core::str::from_utf8(byte).unwrap());
      }
      len as isize
    }
    _ => {
      panic!("Unsupported fd");
    }
  }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
  match fd {
    FD_STDIN => {
      assert_eq!(len, 1, "Only support len = 1 in sys_read");
      let c: usize;
      loop {
        c = console_getchar();
        assert_ne!(c, 0);
        break;
      }
      let ch = c as u8;
      let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
      unsafe {
        buffers[0].as_mut_ptr().write_volatile(ch);
      }
      1
    }
    _ => {
      panic!("Unsupported fd");
    }
  }
}