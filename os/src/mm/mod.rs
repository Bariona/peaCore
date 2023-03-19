use self::heap_allocator::init_heap;

mod address;
mod heap_allocator;
mod frame_allocator;
mod page_table;

pub fn init() {
  init_heap();
}
