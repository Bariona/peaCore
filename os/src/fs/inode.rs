use alloc::{sync::Arc, vec::Vec};
use easy_fs::{Inode, FileSystem};

use crate::sync::UPSafeCell;

pub struct OSInode {
  readable: bool, // immutable info
  writable: bool,
  inner: UPSafeCell<OSInodeInner>, // mutable info
}

pub struct OSInodeInner {
  offset: usize,
  inode: Arc<Inode>,
}

impl OSInode {
  pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
    Self {
      readable,
      writable,
      inner: unsafe {
        UPSafeCell::new(OSInodeInner { offset: 0, inode })
      }
    }
  }

  /// Read all data inside a inode into a vector
  pub fn read_all(&self) -> Vec<u8> {
    let mut inner = self.inner.exclusive_access();
    let mut buf = [0u8; 512];
    let mut vec: Vec<_> = Vec::new();
    loop {
      let len = inner.inode.read_at(inner.offset, &mut buf);
      if len == 0 {
        break;
      }
      inner.offset += len;
      vec.extend_from_slice(&buf);
    }
    vec
  }  
}


lazy_static! {
  // pub static ref ROOT_INODE: Arc<Inode> = {
  //   let fs = FileSystem::open(BLOCK_DEV);
  // }
}
