use core::any::Any;

/// API provided for File System
pub trait BlockDevice: Send + Sync + Any {
  /// read from block data to `buf`
  fn read_block(&self, block_id: usize, buf: &mut [u8]);  
  
  /// write data back to block
  fn write_block(&self, block_id: usize, buf: &[u8]);
}