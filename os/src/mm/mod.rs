use crate::mm::address::VirtAddr;

use self::{heap_allocator::init_heap, frame_allocator::init_frame_allocator, memory_set::KERNEL_SPACE};

pub mod address;
pub mod memory_set;
pub use memory_set::remap_test;
pub use heap_allocator::heap_test;
pub use page_table::{translated_byte_buffer, translated_str, translated_refmut};
mod heap_allocator;
mod frame_allocator;
mod page_table;

pub fn init() {
  init_heap();
  heap_test();
  init_frame_allocator();
  assert!(KERNEL_SPACE.exclusive_access().check_valid(VirtAddr::from(0x1000_0000)));
  KERNEL_SPACE.exclusive_access().activate();
  
  // remap_test();
}
