use core::ptr;

#[derive(Clone, Copy)]
pub struct LinkedList {
  head: *mut usize
}

unsafe impl Send for LinkedList {}

impl LinkedList {
  pub const fn new() -> Self {
    Self { 
      head: ptr::null_mut()
    }
  }

  pub fn is_empty(&self) -> bool {
    self.head.is_null()
  }

  pub unsafe fn push(&mut self, item: *mut usize) {
    *item = self.head as usize;
    self.head = item;
  }

  pub fn pop(&mut self) -> Option<*mut usize> {
    if self.is_empty() {
      return None;
    }
    let ret = self.head;
    self.head = unsafe { *ret as *mut usize };
    Some(ret)
  }

  #[allow(unused)]
  pub fn iter(&self) -> Iter {
    Iter {
      curr: self.head,
      list: self
    }
  }

  pub fn iter_mut(&mut self) -> IterMut {
    IterMut { 
      // a point that points to a pointer
      prev: (&mut self.head as *mut *mut usize) as *mut usize, 
      curr: self.head,
      list: self, 
    }
  }
}

#[allow(unused)]
pub struct Iter<'a> {
  curr: *mut usize,
  list: &'a LinkedList
}

impl Iterator for Iter<'_> {
  type Item = *mut usize;

  fn next(&mut self) -> Option<Self::Item> {
    if self.curr.is_null() {
      None
    } else {
      let cur = self.curr;
      let nex = unsafe { *cur as *mut usize };
      self.curr = nex;
      Some(cur)
    }
  }
}

pub struct ListNode {
  prev: *mut usize,
  curr: *mut usize
}

impl ListNode {
  /// remove node from list
  pub fn pop(&self) -> *mut usize {
    unsafe {
      *(self.prev) = *(self.curr);
    }
    self.curr
  }

  pub fn value(&self) -> *mut usize {
    self.curr
  }
}

#[allow(unused)]
pub struct IterMut<'a> {
  list: &'a mut LinkedList,
  prev: *mut usize, 
  curr: *mut usize
}

impl Iterator for IterMut<'_> {
  type Item = ListNode;

  fn next(&mut self) -> Option<Self::Item> {
    if self.curr.is_null() {
      return None;
    }
    let ret = ListNode {
      prev: self.prev,
      curr: self.curr,
    };
    self.prev = self.curr;
    self.curr = unsafe { *(self.curr) as *mut usize };
    Some(ret)
  }
  
}