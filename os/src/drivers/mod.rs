use alloc::sync::Arc;
use easy_fs::BlockDevice;

mod virtio_blk;

use virtio_blk::VirtIOBlock;

lazy_static! {
  pub static ref BLOCK_DEV: Arc<dyn BlockDevice> = Arc::new(VirtIOBlock::new());
}

#[allow(unused)]
pub fn block_device_test() {
  let block_device = BLOCK_DEV.clone();
  let mut write_buffer = [0u8; 512];
  let mut read_buffer: [u8; 512] = [0u8; 512];
  for i in 0..512 {
      for byte in write_buffer.iter_mut() {
          *byte = i as u8;
      }
      block_device.write_block(i as usize, &write_buffer);
      block_device.read_block(i as usize, &mut read_buffer);
      assert_eq!(write_buffer, read_buffer);
  }
  println!("block device test passed!");
}
