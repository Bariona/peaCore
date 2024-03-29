//! Implementation of [`FrameAllocator`] which
//! controls all the **physical** frames in the operating system.

use alloc::vec::Vec;

use crate::{sync::up::UPSafeCell, mm::address::PhysAddr, config::MEMORY_ENDPOINT};
use core::fmt::{Debug};
use super::address::PhysPageNum;

/// manage a frame which has the same lifecycle as the tracker.
pub struct FrameTracker {
  pub ppn: PhysPageNum
}

impl FrameTracker {
  /// Will clear the page before its usage
  pub fn new(ppn: PhysPageNum) -> Self {
    let bytes_array = ppn.get_bytes_array();
    for i in bytes_array {
      *i = 0;
    }
    Self { ppn }
  }
}

impl Debug for FrameTracker {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_fmt(format_args!("FrameTracker: PPN = {:#x}", self.ppn.0))
  }
}

impl Drop for FrameTracker {
  fn drop(&mut self) {
    frame_dealloc(self.ppn);   
  }
}

/// trait that describes how physcial frames should be managed
trait FrameAllocator {
  fn new() -> Self;
  fn alloc(&mut self) -> Option<PhysPageNum>;
  fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
  // [current, end)
  current: usize,
  end: usize,
  recycled: Vec<usize>
}

impl StackFrameAllocator {
  pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
    self.current = l.0;
    self.end = r.0;
    println!("last {}  Frames.", self.end - self.current);
  }
}

impl FrameAllocator for StackFrameAllocator {
  fn new() -> Self {
    Self {
      current: 0,
      end: 0,
      recycled: Vec::new()
    }
  }

  fn alloc(&mut self) -> Option<PhysPageNum> {
    // println!("{}", self.current);
    if let Some(ppn) = self.recycled.pop() {
      Some(ppn.into())
    } else if self.current == self.end {
      None
    } else {
      let tmp = self.current;
      self.current += 1;
      Some(tmp.into())
    }
  }

  fn dealloc(&mut self, ppn: PhysPageNum) {
    let ppn = ppn.0;
    if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
      panic!("Frame ppn = {:#x} has nott been allocated!", ppn);
    }
    self.recycled.push(ppn);
  }
}

lazy_static! {
  pub static ref FRAME_ALLOCATOR: UPSafeCell<StackFrameAllocator> =
    unsafe {
      UPSafeCell::new(StackFrameAllocator::new())
    };
}

/// initiate the frame allocator using `ekernel` and `MEMORY_END`
pub fn init_frame_allocator() {
  extern "C" {
    fn ekernel();
  }
  FRAME_ALLOCATOR.exclusive_access().init(
    PhysAddr::from(ekernel as usize).ceil(), 
    PhysAddr::from(MEMORY_ENDPOINT).floor()
  )
}

/// allocate a frame and **clear** it
pub fn frame_alloc() -> Option<FrameTracker> {
  FRAME_ALLOCATOR
    .exclusive_access()
    .alloc()
    .map(FrameTracker::new)
}

/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
  FRAME_ALLOCATOR
    .exclusive_access()
    .dealloc(ppn);
}


#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
  let mut v: Vec<FrameTracker> = Vec::new();
  for i in 0..5 {
    let frame = frame_alloc().unwrap();
    println!("{:?}", frame);
    v.push(frame);
  }
  v.clear();
  for i in 0..5 {
    let frame = frame_alloc().unwrap();
    println!("{:?}", frame);
    v.push(frame);
  }
  drop(v);
  println!("frame_allocator_test passed!");
}
