//! Load user applications into memory

/// Get the total number of applications
pub fn get_num_app() -> usize {
  extern "C" {
    fn _num_app();
  }
  // TODO: _num_app as usize ?
  unsafe { (_num_app as *const usize).read_volatile() }
}

/// get applications data
pub fn get_app_data(app_id: usize) -> &'static [u8] {
  extern "C" {
    fn _num_app();
  }
  let num_app_ptr = _num_app as *const usize;
  let num_app = get_num_app();
  assert!(app_id < num_app);
  // TODO: here, num_app_ptr.add(1) means usize := u64
  let app_start = unsafe {
    // read quad
    core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
  };
  unsafe {
    core::slice::from_raw_parts(
      // read byte
      app_start[app_id] as *const u8, 
      app_start[app_id + 1] - app_start[app_id],
    )
  }
}