use self::{heap_allocator::init_heap, frame_allocator::init_frame_allocator, memory_set::KERNEL_SPACE};

pub mod address;
pub mod memory_set;
pub use memory_set::remap_test;
pub use heap_allocator::heap_test;
pub use page_table::translated_byte_buffer;
mod heap_allocator;
mod frame_allocator;
mod page_table;

pub fn init() {
  init_heap();
  init_frame_allocator();
  KERNEL_SPACE.exclusive_access().activate();
  // remap_test();
}
