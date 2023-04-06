//! File and filesystem-related syscalls

use crate::{mm::translated_byte_buffer, task::current_user_token};

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