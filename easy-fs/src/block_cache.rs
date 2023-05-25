use alloc::{sync::Arc, collections::VecDeque};
use spin::Mutex;
use lazy_static::lazy_static;

use crate::{BLOCK_SZ, block_dev::BlockDevice};

const BLOCK_CACHE_SIZE: usize = 16;

pub struct BlockCache {
  /// cached block data
  cache: [u8; BLOCK_SZ],
  /// corresponding block id
  block_id: usize, 
  /// corresponding block device
  block_device: Arc<dyn BlockDevice>,
  /// dirty bit 
  modified: bool,
}

impl BlockCache {
  /// Load a new BlockCache from disk.
  pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
    let mut cache = [0u8; BLOCK_SZ];
    block_device.read_block(block_id, &mut cache);
    Self {
      cache,
      block_id,
      block_device,
      modified: false,
    }
  }

  /// Get the address of an offset inside the cached block data
  pub fn addr_of_offset(&self, offset: usize) -> usize {
      &self.cache[offset] as *const u8 as usize
  }

  /// get ref of data tyep `T` from block cache entry
  pub fn get_ref<T>(&self, offset: usize) -> &T where T: Sized {
    let type_size = core::mem::size_of::<T>();
    assert!(offset + type_size <= BLOCK_SZ);
    unsafe { &*(self.addr_of_offset(offset) as *const T) }
  }

  /// get mut ref of data tyep `T` from block cache entry
  pub fn get_mut<T>(&mut self, offset: usize) -> &mut T where T: Sized {
    let type_size = core::mem::size_of::<T>();
    assert!(offset + type_size <= BLOCK_SZ);
    self.modified = true;
    unsafe { &mut *(self.addr_of_offset(offset) as *mut T) }
  }

  // Map closure `f` onto <T> cache[offset] 
  pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
    f(self.get_ref(offset))
  }

  pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
    f(self.get_mut(offset))
  }

  /// if dirty, write back to block device
  pub fn sync(&mut self) {
    if self.modified {
      self.block_device.write_block(self.block_id, &self.cache);
      self.modified = false;
    }
  }
}

impl Drop for BlockCache {
  fn drop(&mut self) {
    self.sync();      
  }
}

pub struct BlockCacheManager {
  queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>
}

impl BlockCacheManager {
  pub fn new() -> Self {
    Self {
      queue: VecDeque::new()
    }
  }

  /// find `block_id`-th block on `block_device`
  /// and place it on cache
  pub fn get_block_cache(
    &mut self, 
    block_id: usize, 
    block_device: Arc<dyn BlockDevice>
  ) -> Arc<Mutex<BlockCache>> {
    if let Some((_, entry)) = self.queue.iter().find(|entry| entry.0 == block_id)  {
      return entry.clone();
    } else {
      if self.queue.len() == BLOCK_CACHE_SIZE {
        if let Some((idx, _)) =  
          self.queue
          .iter()
          .enumerate()
          .find(|(_, entry)| Arc::strong_count(&entry.1) == 1) {
          self.queue.drain(idx..=idx);
        } else {
          panic!("Run out of Block Cache entries");
        } 

        // if let Some((idx, _)) =  
        //   self.queue
        //   .iter()
        //   .find(|entry| Arc::strong_count(&entry.1) == 1) {
        //   self.queue.drain(idx..=idx);
        // } else {
        //   panic!("Run out of Block Cache entries");
        // } 
      } 
      let cache_entry = Arc::new(Mutex::new(BlockCache::new(block_id, block_device)));
      self.queue.push_back((block_id, cache_entry.clone()));
      cache_entry
    }
  }
}

lazy_static! {
  pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> = Mutex::new(BlockCacheManager::new());
}

/// Get the block cache corresponding to the given block id and block device
pub fn get_block_cache(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Arc<Mutex<BlockCache>> {
  BLOCK_CACHE_MANAGER.lock().get_block_cache(block_id, block_device)
}

/// Sync all block cache to block device
pub fn block_cache_sync_all() {
  let manager = BLOCK_CACHE_MANAGER.lock();
  manager.queue.iter().for_each(|(_, cache)| {
    cache.lock().sync();
  });
}