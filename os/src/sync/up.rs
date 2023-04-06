//! Uniprocessor interior mutability primitives

use core::cell::{RefCell, RefMut};

/// Motivation: if we want to declare a variable as `static mut`,
/// all of its access will be regarded as `unsafe`, thus we want to avoid it.
/// 
/// Wrap a static data structure inside it so that we are
/// able to access it without any `unsafe`.
///
/// We should only use it in uniprocessor.
///
/// In order to get mutable reference of inner data, call
/// `exclusive_access`.

pub struct UPSafeCell<T> {
  inner: RefCell<T>
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
  /// Programmer is responsible to guarantee that inner struct is only used in uniprocessor.
  pub unsafe fn new(value: T) -> Self {
    Self { inner: RefCell::new(value) }
  }

  /// more strict that `RefCell<T>` since we not only guarantee that there 
  /// 
  /// will be at most one thing that can modify it but also 
  /// 
  /// one that can read it.
  pub fn exclusive_access(&self) -> RefMut<'_, T> {
    self.inner.borrow_mut()
  }
}
