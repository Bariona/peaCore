//! File and filesystem-related syscalls
use crate::{mm::{translated_byte_buffer, UserBuffer, translated_str}, task::processor::{current_user_token, current_task}, fs::{open_file, Flags}};

/// write buf of length `len` to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
  let current_task = current_task().unwrap();
  let inner = &current_task.inner_exclusive_access();

  if fd >= inner.fd_table.len() {
    return -1;
  }
  let user_buf = UserBuffer::new(
    translated_byte_buffer(inner.get_user_token(), buf, len)
  );
  
  if let Some(file) = &inner.fd_table[fd] {
    drop(inner);
    if !file.writable() {
      return -1;
    }
    file.write(user_buf) as isize 
  } else {
    -1
  }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
  let current_task = current_task().unwrap();
  let inner = &current_task.inner_exclusive_access();

  if fd >= inner.fd_table.len() {
    return -1;
  }
  let user_buf = UserBuffer::new(
    translated_byte_buffer(inner.get_user_token(), buf, len)
  );
  
  if let Some(file) = &inner.fd_table[fd] {
    drop(inner);
    if !file.readable() {
      return -1;
    }
    file.read(user_buf) as isize 
  } else {
    -1
  }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
  let current_task = current_task().unwrap();
  let token = current_user_token();
  let path = translated_str(token, path);
  if let Some(inode) = open_file(path.as_str(), Flags::from_bits(flags).unwrap()) {
    let mut inner = current_task.inner_exclusive_access();
    let fd = inner.alloc_fd();
    inner.fd_table[fd] = Some(inode);
    drop(inner);
    fd as isize
  } else {
    -1
  }
}

pub fn sys_close(fd: usize) -> isize {
  let task = current_task().unwrap();
  let mut inner = task.inner_exclusive_access();
  if fd >= inner.fd_table.len() {
    return -1;
  }
  if inner.fd_table[fd].is_none() {
    return -1;
  }
  inner.fd_table[fd].take();
  0
}