use core::{alloc::Layout, mem::size_of};

use crate::ds::buddy::Heap;

pub mod buddy;
mod linked_list;

fn test_linked_list() {
  let mut value1: usize = 0;
  let mut value2: usize = 0;
  let mut value3: usize = 0;
  let mut list = linked_list::LinkedList::new();
  unsafe {
    list.push(&mut value1 as *mut usize);
    list.push(&mut value2 as *mut usize);
    list.push(&mut value3 as *mut usize);
  }

  // Test links
  assert_eq!(value3, &value2 as *const usize as usize);
  assert_eq!(value2, &value1 as *const usize as usize);
  assert_eq!(value1, 0);

  // Test iter
  let mut iter = list.iter();
  assert_eq!(iter.next(), Some(&mut value3 as *mut usize));
  assert_eq!(iter.next(), Some(&mut value2 as *mut usize));
  assert_eq!(iter.next(), Some(&mut value1 as *mut usize));
  assert_eq!(iter.next(), None);

  // Test iter_mut
  let mut iter_mut = list.iter_mut();
  assert_eq!(iter_mut.next().unwrap().pop(), &mut value3 as *mut usize);

  // Test pop
  assert_eq!(list.pop(), Some(&mut value2 as *mut usize));
  assert_eq!(list.pop(), Some(&mut value1 as *mut usize));
  assert_eq!(list.pop(), None);
}

fn test_empty_heap() {
  let a: Heap<32> = Heap::empty();
  let mut heap = Heap::<32>::new();
  assert!(heap.alloc(Layout::from_size_align(1, 1).unwrap()).is_err());
}

fn test_heap_add() {
  let mut heap = Heap::<32>::new();
  assert!(heap.alloc(Layout::from_size_align(1, 1).unwrap()).is_err());

  let space: [usize; 100] = [0; 100];
  unsafe {
    heap.add_to_heap(space.as_ptr() as usize, space.as_ptr().add(100) as usize);
  }
  let addr = heap.alloc(Layout::from_size_align(1, 1).unwrap());
  assert!(addr.is_ok());
}

fn test_heap_oom() {
  let mut heap = Heap::<32>::new();
  let space: [usize; 100] = [0; 100];
  unsafe {
    heap.add_to_heap(space.as_ptr() as usize, space.as_ptr().add(100) as usize);
  }

  assert!(heap
    .alloc(Layout::from_size_align(100 * size_of::<usize>(), 1).unwrap())
    .is_err());
  assert!(heap.alloc(Layout::from_size_align(1, 1).unwrap()).is_ok());
}

fn test_heap_alloc_and_free() {
  let mut heap = Heap::<32>::new();
  assert!(heap.alloc(Layout::from_size_align(1, 1).unwrap()).is_err());

  let space: [usize; 100] = [0; 100];
  unsafe {
    heap.add_to_heap(space.as_ptr() as usize, space.as_ptr().add(100) as usize);
  }
  for _ in 0..100 {
    let addr = heap.alloc(Layout::from_size_align(1, 1).unwrap()).unwrap();
    heap.dealloc(addr, Layout::from_size_align(1, 1).unwrap());
  }
}

pub fn test() {
  test_linked_list();
  test_empty_heap();
  test_heap_add();
  test_heap_oom();
  test_heap_alloc_and_free();
  println!("Data Structure test: \x1b[92m[passed!]\x1b[0m");
}