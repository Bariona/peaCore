use alloc::sync::Arc;

use crate::{BLOCK_SZ, block_dev::BlockDevice, block_cache::get_block_cache};

/// a bit map block
type BitmapBlock = [u64; 64];

const BLOCK_BITS: usize = BLOCK_SZ * 8;

pub struct Bitmap {
  start_block_id: usize, 
  blocks: usize, // total block number
}

fn decompose(mut bit: usize) -> (usize, usize, usize) {
  let blk_id = bit / BLOCK_BITS;
  bit %= BLOCK_BITS;
  (blk_id, bit / 64, bit % 64)
}

impl Bitmap {
  /// A new bitmap from start block id and number of blocks
  pub fn new(start_block_id: usize, blocks: usize) -> Self {
    Self { 
      start_block_id, 
      blocks 
    }
  }    

  /// returns index of an avaliable empty block
  pub fn alloc(&self, block_dev: &Arc<dyn BlockDevice>) -> Option<usize> {
    for block_id in 0..self.blocks {
      let blk_pos = get_block_cache(
        block_id + self.start_block_id, 
        block_dev.clone()
      )
      .lock()
      .modify(0, |bitmap_blk: &mut BitmapBlock| {
        if let Some((bits64_pos, inner_pos)) = 
          bitmap_blk.iter()
          .enumerate()
          .find(|(_, num)| **num != u64::MAX)
          .map(|(idx, num)| (idx, num.trailing_ones())) {
            bitmap_blk[bits64_pos] |= 1u64 << inner_pos;
            Some(block_id * BLOCK_BITS + bits64_pos * 64 + inner_pos as usize)
        } else {
          None
        }
      });
      if blk_pos.is_some() {
        return blk_pos;
      }
    }
    None
  }

  /// Deallocate a block's index
  pub fn dealloc(&self, block_dev: &Arc<dyn BlockDevice>, bit: usize) {
    let (blk_id, bits64_pos, inner_pos) = decompose(bit);
    get_block_cache(blk_id + self.start_block_id, block_dev.clone())
    .lock()
    .modify(0, |bitmap_blk: &mut BitmapBlock| {
      assert_eq!(1, bitmap_blk[bits64_pos] >> inner_pos & 1);
      bitmap_blk[bits64_pos] ^= 1u64 << inner_pos;
    });
  }
  /// Get the max number of allocatable blocks
  pub fn maximum(&self) -> usize {
    self.blocks * BLOCK_BITS
  }
}