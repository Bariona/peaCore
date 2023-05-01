//! Load user applications into memory

use alloc::vec::Vec;

lazy_static! {
  static ref APP_NAMES: Vec<&'static str> = {
    let num_app = get_num_app();
    extern "C" {
      fn _app_names();
    }
    let mut start = _app_names as usize as *const u8;
    let v = Vec::new();
    unsafe {
      for _ in 0..num_app {
        let mut end = start;
        while end.read_volatile() != b'\0' {
          end = end.add(1);
        }
        let slice = core::slice::from_raw_parts(start, end - start);
        let str = core::str::from_utf8(slice);
        v.push(str);
        start = end.add(1);
      }
    }
    v
  };
}
/// Get the total number of applications
pub fn get_num_app() -> usize {
  extern "C" {
    fn _num_app();
  }
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

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
  let num_app = get_num_app();
  (0..num_app)
    .find(|&i| APP_NAMES[i] == name)
    .map(get_app_data)
}

pub fn list_apps() {
  println!("==== BEGIN: APP List ====");
  let app_num = get_num_app();
  for i in 0..app_num {
    println!("{}", APP_NAMES[i]);
  }
  println!("==== END of List ====")
}