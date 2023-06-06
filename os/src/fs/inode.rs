use alloc::{sync::Arc, vec::Vec};
use bitflags::bitflags;
use easy_fs::{Inode, FileSystem};

use crate::{sync::UPSafeCell, drivers::BLOCK_DEV};

use super::File;

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

  /// Read all data (as bytes) inside a inode into a vector
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
  pub static ref ROOT_INODE: Arc<Inode> = {
    let fs = FileSystem::open(BLOCK_DEV.clone());
    Arc::new(FileSystem::root_inode(&fs))
  };
}

pub fn list_apps() {
  let app_list = ROOT_INODE.ls();
  println!("==== BEGIN: APP List ====");
  for app in app_list {
    println!("{}", app);
  }
  println!("==== END of List ====");
}

bitflags! {
  pub struct Flags: u32 {
    const RDONLY = 0;
    const WRONLY = 1 << 0;
    /// read & write
    const RDWR = 1 << 1;
    /// if the file doesn't exist, create it
    const CREATE = 1 << 9;
    /// clear file and return an empty one
    const TRUNC = 1 << 10;
  }
}

impl Flags {
  /// return (readable, writable)
  pub fn rdwr_flags(&self) -> (bool, bool) {
    if self.is_empty() {
      (true, false)
    } else if self.contains(Flags::WRONLY) {
      (false, true)
    } else {
      (true, true)
    }
  }
}

pub fn open_file(name: &str, flags: Flags) -> Option<Arc<OSInode>> {
  let (readable, writable) = flags.rdwr_flags();
  if let Some(inode) = ROOT_INODE.find_name(name) {
    if flags.contains(Flags::TRUNC) {
      inode.clear();
    }
    Some(Arc::new(OSInode::new(readable, writable, inode)))
  } else if flags.contains(Flags::CREATE) {
    let inode = ROOT_INODE.create(name).unwrap();
    Some(Arc::new(OSInode::new(readable, writable, inode)))
  } else {
    None
  }
}

impl File for OSInode {
  fn readable(&self) -> bool {
    self.readable
  }

  fn writable(&self) -> bool {
    self.writable
  }

  fn read(&self, mut buf: crate::mm::UserBuffer) -> usize {
    let mut inner = self.inner.exclusive_access();
    let start = inner.offset;
    for buf in buf.buffers.iter_mut() {
      let size = inner.inode.read_at(inner.offset, buf);
      inner.offset += size;
      if size == 0 {
        break;
      }
    }
    inner.offset - start
  }

  fn write(&self, buf: crate::mm::UserBuffer) -> usize {
    let mut inner = self.inner.exclusive_access();
    let start = inner.offset;
    for buf in buf.buffers.iter() {
      let size = inner.inode.write_at(inner.offset, buf);
      assert_eq!(size, buf.len());
      inner.offset += size;
    }
    inner.offset - start
  }
}

