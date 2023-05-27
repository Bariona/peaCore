use clap::{App, Arg};
use easy_fs::{BlockDevice, FileSystem};
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::sync::Mutex;

use easy_fs::BLOCK_SZ;

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
  fn read_block(&self, block_id: usize, buf: &mut [u8]) {
    let mut file = self.0.lock().unwrap();
    file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
        .expect("Error occurred when seeking");
    assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
  }

  fn write_block(&self, block_id: usize, buf: &[u8]) {
    let mut file = self.0.lock().unwrap();
    file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
        .expect("Error when seeking!");
    assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
  }
}
pub fn main() {

}



#[test]
fn efs_test() -> std::io::Result<()> {
  let block_file = Arc::new(BlockFile(Mutex::new({
    let f = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open("target/fs.img")?;
    f.set_len(8192 * 512).unwrap();
    f
  })));
  FileSystem::create(block_file.clone(), 4096, 1);
  let efs = FileSystem::open(block_file.clone());
  let root_inode = FileSystem::root_inode(&efs);
  
  root_inode.create("filea");
  root_inode.create("fileb");
  
  for name in root_inode.ls() {
    println!("{}", name);
  }
  let filea = root_inode.find_name("filea").unwrap();
  let greet_str = "Hello, world!";
  filea.write_at(0, greet_str.as_bytes());
  
  //let mut buffer = [0u8; 512];
  let mut buffer = [0u8; 233];
  let len = filea.read_at(0, &mut buffer);
  assert_eq!(greet_str, core::str::from_utf8(&buffer[..len]).unwrap(),);

  let mut random_str_test = |len: usize| {
    filea.clear();
    
    assert_eq!(filea.read_at(0, &mut buffer), 0, "not cleared!");
    let mut str = String::new();
    
    use rand;
    // random digit
    for _ in 0..len {
      str.push(char::from('0' as u8 + rand::random::<u8>() % 10));
    }
    filea.write_at(0, str.as_bytes());
    let mut read_buffer = [0u8; 127];
    let mut offset = 0usize;
    let mut read_str = String::new();
    loop {
      let len = filea.read_at(offset, &mut read_buffer);
      if len == 0 {
        break;
      }
      offset += len;
      read_str.push_str(core::str::from_utf8(&read_buffer[..len]).unwrap());
    }
    assert_eq!(str, read_str);
  };

  println!("0");
  random_str_test(4 * BLOCK_SZ);
  println!("phase 1:");
  random_str_test(8 * BLOCK_SZ + BLOCK_SZ / 2);
  println!("phase 2:");
  random_str_test(100 * BLOCK_SZ);
  random_str_test(70 * BLOCK_SZ + BLOCK_SZ / 7);
  random_str_test((12 + 128) * BLOCK_SZ);
  println!("phase 3:");
  random_str_test(400 * BLOCK_SZ);
  println!("phase 4:");
  random_str_test(1000 * BLOCK_SZ);
  random_str_test(2000 * BLOCK_SZ);

  Ok(())
}
