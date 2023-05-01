
use core::{mem::size_of, cmp::{min, max}, alloc::{Layout, GlobalAlloc}, ptr::NonNull, ops::Deref};
use spin::Mutex;
use super::linked_list::LinkedList;

pub struct Heap<const ORDER: usize> {
  free_list: [LinkedList; ORDER],

  user: usize,
  allocated: usize,
  total: usize
}

impl<const ORDER: usize> Heap<ORDER> {
  /// return a empty heap
  pub const fn new() -> Self {
    Self {  
      free_list: [LinkedList::new(); ORDER],
      user: 0,
      allocated: 0,
      total: 0,
    }
  }

  pub const fn empty() -> Self {
    Self::new()
  }

  /// init heap allocator with [start, start + len)
  pub unsafe fn init(&mut self, start: usize, len: usize) {
    self.add_to_heap(start, start + len);
  }

  /// add available memeory [start, end) to heap
  pub unsafe fn add_to_heap(&mut self, mut start: usize, mut end: usize) {
    // bravo implementation of alignment!
    start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
    end = end & (!size_of::<usize>() + 1);
    assert!(start <= end);

    let mut total: usize = 0;
    let mut current_start: usize = start;

    while current_start + size_of::<usize>() <= end {
      let lowbit = current_start &  (!current_start + 1);
      let size = min(lowbit, prev_power_of_two(end - current_start));
      total += size;
      self.free_list[size.trailing_zeros() as usize].push(current_start as *mut usize);
      current_start += size;
    }
    self.total += total;
  }

  /// Alloc a range of memory from the heap satifying `layout` requirements
  pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()>{
    let size = max(
      layout.size().next_power_of_two(),
      max(layout.align(), size_of::<usize>())
    );
    let class = size.trailing_zeros() as usize;
    for i in class..self.free_list.len() {
      if self.free_list[i].is_empty() {
        continue;
      }
      // [class + 1 <- i]
      for j in (class + 1..i + 1).rev() {
        if let Some(block) = self.free_list[j].pop() {
          unsafe {
            self.free_list[j - 1].push(((block as usize) + (1 << (j - 1))) as *mut usize);
            self.free_list[j - 1].push(block);
          }
        } else {
          return Err(())
        }
      }

      // println!("size = {}, class = {}, remaning = {}, allocated = {}, i = {}, j = {}, is_empty = {}", 
      //         size, class, self.total, self.allocated, i, j, self.free_list[class].is_empty());
      let result = NonNull::new(
        self.free_list[class]
          .pop()
          .expect("current block should have free space now")
          as *mut u8
      );
      if let Some(result) = result {
        self.user += layout.size();
        self.allocated += size;
        return Ok(result);
      } else {
        return Err(())
      }
      
    }
    Err(())
  }

  /// Dealloc a range of memory from the heap
  pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
    let size = max(
      layout.size().next_power_of_two(),
      max(layout.align(), size_of::<usize>())
    );
    let class = size.trailing_zeros() as usize;
    unsafe {
      // place it back
      self.free_list[class].push(ptr.as_ptr() as *mut usize);
      let mut current_ptr = ptr.as_ptr() as usize;
      let mut current_class = class;
      
      while current_class < self.free_list.len() {
        let buddy = current_ptr ^ (1 << current_class);
        let mut find_flag = false;
        for block in self.free_list[current_class].iter_mut() {
          if block.value() as usize == buddy {
            block.pop();
            find_flag = true;
            break;
          }
        }

        // free it
        if find_flag {
          self.free_list[current_class].pop();
          current_ptr = min(buddy, current_ptr);
          current_class += 1;
          self.free_list[current_class].push(current_ptr as *mut usize);
        } else {
          break;
        }
      }
    }
    self.user -= layout.size();
    self.allocated -= layout.align();
  }
}


/// biggest power of two than <= num
pub fn prev_power_of_two(num: usize) -> usize {
  1 << (8 * (size_of::<usize>()) - num.leading_zeros() as usize - 1)
}

pub struct LockedHeap<const ORDER: usize> (Mutex<Heap<ORDER>>);

unsafe impl<const ORDER: usize> GlobalAlloc for LockedHeap<ORDER> {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    self.0
      .lock()
      .alloc(layout)
      .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    self.0
      .lock()
      .dealloc(NonNull::new_unchecked(ptr), layout)
  }
}

impl<const ORDER: usize> LockedHeap<ORDER> {
  #[allow(unused)]
  pub const fn new() -> Self {
    LockedHeap(Mutex::new(Heap::new()))
  }

  pub const fn empty() -> Self {
    LockedHeap(Mutex::new(Heap::empty()))
  }
}

impl<const ORDER: usize> Deref for LockedHeap<ORDER> {
  type Target = Mutex<Heap<ORDER>>;

  fn deref(&self) -> &Mutex<Heap<ORDER>> {
    &self.0
  }
}