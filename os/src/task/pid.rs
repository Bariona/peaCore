use alloc::vec::Vec;

use crate::{sync::up::UPSafeCell, config::{TRAMPOLINE, KERNEL_STACK_SIZE, PAGE_SIZE}, mm::{memory_set::{KERNEL_SPACE, MapPermission}, address::VirtAddr}};


pub struct PidHandler(pub usize);

impl Drop for PidHandler {
  fn drop(&mut self) {
    PID_ALLOCATOR.exclusive_access().dealloc(self);
  }
}
 
pub struct PidAllocator {
  current: usize,
  recycled: Vec<usize>
}

impl PidAllocator {
  pub fn new() -> Self {
    Self {
      current: 0,
      recycled: Vec::new(),
    }
  }

  /// Allocate pid for process
  pub fn alloc(&mut self) -> PidHandler {
    if let Some(pid) = self.recycled.pop() {
      PidHandler(pid)
    } else {
      self.current += 1;
      PidHandler(self.current - 1)
    }
  }

  /// Recyle process's pid
  pub fn dealloc(&mut self, pid: &PidHandler) {
    assert!(pid.0 < self.current);
    assert!(!self.recycled.iter().any(|p| pid.0 == *p), "pid {} has been deallocated!", pid.0);
    self.recycled.push(pid.0);
  }
}

lazy_static!{
  pub static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> = unsafe {
    UPSafeCell::new(PidAllocator::new())
  };
}

pub fn pid_alloc() -> PidHandler {
  PID_ALLOCATOR.exclusive_access().alloc()
}

/// return process's kernel stack layout: (bottom, top)
pub fn kernel_stack_position(pid: usize) -> (usize, usize) {
  let top = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE);
  let bottom = top - KERNEL_STACK_SIZE;
  (bottom, top)
}

/// Kernel stack for app 
pub struct KernelStack {
  pid: usize
}

impl KernelStack {
  /// Alloc Kernel Stack of the corresponding `pid` (modify PageTable)
  pub fn new(pid_handler: PidHandler) -> Self {
    let pid = pid_handler.0;
    let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
    KERNEL_SPACE.exclusive_access().insert_framed_area(
      kernel_stack_bottom.into(),
      kernel_stack_top.into(), 
      MapPermission::R | MapPermission::W
    );
    Self { pid }
  }

  /// Returns the position of the top of kernelstack
  pub fn get_top(&self) -> usize {
    let (_, kernel_stack_top) = kernel_stack_position(self.pid);
    kernel_stack_top
  }
}

impl Drop for KernelStack {
  fn drop(&mut self) {
    let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
    let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
    KERNEL_SPACE
      .exclusive_access()
      .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
  }
}