// use core::ptr;


// #[derive(Clone, Copy)]
// pub struct LinkedList {
//   head: *mut usize
// }

// impl LinkedList {
//   pub const fn new() -> Self {
//     Self { 
//       head: ptr::null_mut()
//     }
//   }

//   pub fn is_empty(&self) -> bool {
//     self.head.is_null()
//   }

//   pub unsafe fn push(&mut self, item: *mut usize) {
//     *item = self.head as usize;
//     self.head = item;
//   }

//   pub fn pop(&mut self) -> Option<*mut usize> {
//     if self.is_empty() {
//       return None;
//     }
//     let ret = self.head;
//     self.head = unsafe { *ret as *mut usize };
//     Some(ret)
//   }

//   pub fn iter(&self) -> Iter {

//   }

// }