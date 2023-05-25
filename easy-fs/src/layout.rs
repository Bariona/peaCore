//! Layout of FileSystem Structure
use core::{fmt::{Debug, Result, Formatter}, mem::size_of, cmp::min};

use alloc::{sync::Arc, vec::Vec};

use crate::{BLOCK_SZ, block_dev::BlockDevice, block_cache::get_block_cache, DataBlock};

/// FileSystem Magic Number
const FS_MAGIC: u32 = 0x3b800001;
/// Inode direct index
const INODE_DIRECT_COUNT: usize = 28;
/// indirect index range
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4;
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * BLOCK_SZ / 4;
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;

const NAME_LENGTH_LIMIT: usize = 27;

/// Block that stores indirect block's indexes
type IndirectBlock = [u32; BLOCK_SZ / size_of::<u32>()];

/// Super block of a filesystem
#[repr(C)]
pub struct SuperBlock {
  magic: u32,
  pub total_blocks: u32,
  pub inode_bitmap_blocks: u32,
  pub inode_area_blocks: u32,
  pub data_bitmap_blocks: u32,
  pub data_area_blocks: u32,
}

impl Debug for SuperBlock {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.debug_struct("SuperBlock")
      .field("total_blocks", &self.total_blocks)
      .field("inode_bitmap_blocks", &self.inode_bitmap_blocks)
      .field("inode_area_blocks", &self.inode_area_blocks)
      .field("data_bitmap_blocks", &self.data_bitmap_blocks)
      .field("data_area_blocks", &self.data_area_blocks)
      .finish()
  }
}

impl SuperBlock {
  pub fn new(
    total_blocks: u32,
    inode_bitmap_blocks: u32,
    inode_area_blocks: u32,
    data_bitmap_blocks: u32,
    data_area_blocks: u32,
  ) -> Self {
    Self {
      magic: FS_MAGIC,
      total_blocks,
      inode_bitmap_blocks,
      inode_area_blocks,
      data_bitmap_blocks,
      data_area_blocks,
    }
  }

  pub fn is_valid(&self) -> bool {
    self.magic == FS_MAGIC
  }
}


#[derive(PartialEq)]
pub enum DiskInodeType {
  File, 
  Directory,
}

#[repr(C)]
pub struct DiskInode {
  /// file's total bytes
  size: u32, 
  direct: [u32; INODE_DIRECT_COUNT],
  indirect1: u32,
  indirect2: u32,
  type_: DiskInodeType,
}

impl DiskInode {
  pub fn initialize(&mut self, type_: DiskInodeType) {
    self.size = 0;
    self.direct.iter_mut().for_each(|a| *a = 0);
    self.indirect1 = 0;
    self.indirect2 = 0;
    self.type_ = type_;
  }

  pub fn is_dir(&self) -> bool {
    self.type_ == DiskInodeType::Directory
  } 

  pub fn is_file(&self) -> bool {
    self.type_ == DiskInodeType::File
  }

  /// blocks number for storing data (dispite of inodes)
  pub fn data_blocks(&self) -> u32 { 
    DiskInode::_data_blocks(self.size)
  }
  pub fn _data_blocks(size: u32) -> u32 {
    (size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32
  }

  /// Return number of blocks needed include indirect1/2.
  pub fn total_blocks(size: u32) -> u32 {
    let data_blocks = Self::_data_blocks(size) as usize;
    let mut total = data_blocks as usize;
    // TODO: total += 1 ?
    if data_blocks > INODE_DIRECT_COUNT {
      total += 1;
    }
    if data_blocks > INDIRECT1_BOUND {
      total += 1;
      total += (data_blocks - INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1) / INODE_INDIRECT1_COUNT;
    }
    total as u32
  }

  pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
    assert!(new_size >= self.size);
    Self::total_blocks(new_size) - Self::total_blocks(self.size)
  }

  pub fn get_block_id(&self, inner_id: u32, block_dev: &Arc<dyn BlockDevice>) -> u32 {
    let inner_id = inner_id as usize;
    if inner_id < DIRECT_BOUND {
      self.direct[inner_id]
    } else if inner_id < INDIRECT1_BOUND {
      get_block_cache(self.indirect1 as usize, block_dev.clone())
        .lock()
        .read(0, |indirect1: &IndirectBlock| {
          indirect1[inner_id - DIRECT_BOUND]
        }) 
    } else {
      let indirect1 = get_block_cache(self.indirect2 as usize, block_dev.clone())
        .lock()
        .read(0, |indirect_blks: &IndirectBlock| {
          indirect_blks[(inner_id - INDIRECT1_BOUND) / INODE_INDIRECT1_COUNT]
        });
      get_block_cache(indirect1 as usize, block_dev.clone())
        .lock()
        .read(0, |indirect2: &IndirectBlock| {
          indirect2[(inner_id - INDIRECT1_BOUND) % INODE_INDIRECT1_COUNT]
        })
    }
  }

  /// increase current file's size to `new_size`
  pub fn increase_size(
    &mut self,
    new_size: u32,
    new_blocks: Vec<u32>,
    block_dev: &Arc<dyn BlockDevice>,
  ) {
    assert!(new_size >= self.size);
    let mut cur_data_blks = self.data_blocks() as usize;
    self.size = new_size;
    let mut tot_data_blks = self.data_blocks() as usize;
    
    let mut blk_iter = new_blocks.into_iter();
    while cur_data_blks < min(DIRECT_BOUND, tot_data_blks) {
      self.direct[cur_data_blks] = blk_iter.next().unwrap();
      cur_data_blks += 1;
    }

    if tot_data_blks <= DIRECT_BOUND {
      return ;
    } 

    assert!(cur_data_blks >= DIRECT_BOUND);
    if cur_data_blks == INODE_DIRECT_COUNT { // expand indirect1
      self.indirect1 = blk_iter.next().unwrap();
    }
    cur_data_blks -= DIRECT_BOUND;
    tot_data_blks -= DIRECT_BOUND;

    if cur_data_blks < min(INODE_INDIRECT1_COUNT, tot_data_blks) {
      get_block_cache(self.indirect1 as usize, block_dev.clone())
        .lock()
        .modify(0, |indirect_blks: &mut IndirectBlock| {
          while cur_data_blks < min(INODE_INDIRECT1_COUNT, tot_data_blks) {
            indirect_blks[cur_data_blks] = blk_iter.next().unwrap();
            cur_data_blks += 1;
          }
        });
    }
    
    if tot_data_blks <= INODE_INDIRECT1_COUNT {
      return;
    }

    assert!(cur_data_blks >= INODE_INDIRECT1_COUNT);
    if cur_data_blks == INODE_INDIRECT1_COUNT {
      self.indirect2 = blk_iter.next().unwrap();
    }
    cur_data_blks -= INODE_INDIRECT1_COUNT;
    tot_data_blks -= INODE_INDIRECT1_COUNT;

    let mut a0 = cur_data_blks / INODE_INDIRECT1_COUNT;
    let mut b0 = cur_data_blks % INODE_INDIRECT1_COUNT;
    let a1 = tot_data_blks / INODE_INDIRECT1_COUNT;
    let b1 = tot_data_blks / INODE_INDIRECT1_COUNT;
    get_block_cache(self.indirect2 as usize, block_dev.clone())
      .lock()
      .modify(0, |indirect1: &mut IndirectBlock| {
        while a0 <= a1 {
          while (a0 < a1) || (a0 == a1 && b0 < b1) {
            if b0 == 0 {
              indirect1[a0] = blk_iter.next().unwrap();
            }
            // modify (a0, b0)
            get_block_cache(indirect1[a0] as usize, block_dev.clone())
              .lock()
              .modify(0, |indirect2: &mut IndirectBlock| {
                indirect2[b0] = blk_iter.next().unwrap();
              }); 
            b0 += 1;
            if b0 == INODE_INDIRECT1_COUNT {
              b0 = 0;
              a0 += 1;
            }
          }
        }
      });
  }

  /// Clear size to zero and return blocks that should be deallocated.
  /// We will clear the block contents to zero later.
  /// TODO: debug
  pub fn clear_size(&mut self, block_dev: &Arc<dyn BlockDevice>) -> Vec<u32> {
    let mut vec: Vec<u32> = Vec::new();
    let mut tot_data_blks = self.data_blocks() as usize;
    let mut cur_data_blks = 0usize;

    self.size = 0;

    while cur_data_blks < min(DIRECT_BOUND, tot_data_blks){
      vec.push(self.direct[cur_data_blks]);
      self.direct[cur_data_blks] = 0;
      cur_data_blks += 1;
    } 

    if tot_data_blks <= DIRECT_BOUND {
      return vec;
    }

    vec.push(self.indirect1);
    tot_data_blks -= DIRECT_BOUND;
    cur_data_blks -= DIRECT_BOUND;
    
    get_block_cache(self.indirect1 as usize, block_dev.clone())
      .lock()
      .modify(0, |indirect_blks: &mut IndirectBlock| {
        while cur_data_blks < min(INODE_INDIRECT1_COUNT, tot_data_blks) {
          vec.push(indirect_blks[cur_data_blks]);
          indirect_blks[cur_data_blks] = 0;
          cur_data_blks += 1;
        }
      });
    self.indirect1 = 0;

    if tot_data_blks <= INODE_INDIRECT1_COUNT {
      return vec;
    }
    vec.push(self.indirect2);
    tot_data_blks -= INODE_INDIRECT1_COUNT;
    cur_data_blks -= INODE_INDIRECT1_COUNT;

    get_block_cache(self.indirect2 as usize, block_dev.clone())
      .lock()
      .modify(0, |indirect1: &mut IndirectBlock| {
        let mut retrived: usize = 0;
        while retrived * INODE_INDIRECT1_COUNT <= tot_data_blks {
          get_block_cache(indirect1[retrived] as usize, block_dev.clone())
            .lock()
            .modify(0, |indirect2: &mut IndirectBlock| {
              let mut i: usize = 0;
              while cur_data_blks <= min(INODE_INDIRECT1_COUNT, tot_data_blks) {
                vec.push(indirect2[i]);
                i += 1;
                cur_data_blks += 1;
              }
            });
          vec.push(indirect1[retrived]);
          retrived += 1;
        }
      });
    self.indirect2 = 0;

    vec
  }

  /// read data from disk inode to `buf`
  pub fn read_at(&self, offset: usize, buf: &mut [u8], block_dev: &Arc<dyn BlockDevice>) -> usize {
    // [start, end)
    let mut start = offset;
    let end = min(self.size as usize, start + buf.len());
    if start >= end {
      return 0;
    }
    let mut start_block = start / BLOCK_SZ;
    let mut read_size = 0usize;
    loop {
      let cur_block_end = min(end, (start / BLOCK_SZ + 1) * BLOCK_SZ);
      let block_read_size = cur_block_end - start;
      let dst = &mut buf[read_size..read_size + block_read_size];
      get_block_cache(
        self.get_block_id(start_block as u32, block_dev) as usize, 
        block_dev.clone()
      ) .lock()
        .read(0, |data: &DataBlock| {
          assert!(start % BLOCK_SZ + block_read_size < BLOCK_SZ);
          let src = &data[start % BLOCK_SZ..start % BLOCK_SZ + block_read_size];
          dst.copy_from_slice(src);
        });

      read_size += block_read_size;
      start += block_read_size;
      start_block += 1;
      if end == cur_block_end {
        break;
      }
    }
    read_size
  }

  /// write data into disk inode from `buf`
  pub fn write_at(&mut self, offset: usize, buf: &[u8], block_dev: &Arc<dyn BlockDevice>) -> usize {
    let mut start = offset;
    let end = min(self.size as usize, start + buf.len());
    if start >= end {
      return 0;
    }
    let mut start_block = start / BLOCK_SZ;
    let mut write_size = 0usize;
    loop {
      let cur_block_end = min(end, (start / BLOCK_SZ + 1) * BLOCK_SZ);
      let block_read_size = cur_block_end - start;
      let src = &buf[write_size..write_size + block_read_size];
      get_block_cache(
        self.get_block_id(start_block as u32, block_dev) as usize, 
        block_dev.clone()
      ) .lock()
        .modify(0, |data: &mut DataBlock| {
          assert!(start % BLOCK_SZ + block_read_size < BLOCK_SZ);
          let dst = &mut data[start % BLOCK_SZ..start % BLOCK_SZ + block_read_size];
          dst.copy_from_slice(src);
        });
      write_size += block_read_size;
      start += block_read_size;
      start_block += 1;
      if end == cur_block_end {
        break;
      }
    }
    write_size
  }

}

#[repr(C)]
pub struct DirEntry {
  name: [u8; NAME_LENGTH_LIMIT + 1],
  inode: u32,
}

/// size of a directory entry
pub const DIRENT_SZ: usize = 32;

impl DirEntry {
  pub fn empty() -> Self {
    Self { 
      name: [0; NAME_LENGTH_LIMIT + 1], 
      inode: 0
    }
  }

  pub fn new(name: &str, inode: u32) -> Self {
    let mut bytes = [0u8; NAME_LENGTH_LIMIT + 1];
    bytes.copy_from_slice(name.as_bytes());
    Self { 
      name: bytes,
      inode 
    }
  }

  /// Serialize into bytes
  pub fn as_bytes(&self) -> &[u8] {
    unsafe { 
      core::slice::from_raw_parts(self as *const _ as usize as *const u8, DIRENT_SZ) 
    }
  }

  /// Serialize into mutable bytes
  pub fn as_bytes_mut(&mut self) -> &mut [u8] {
    unsafe { 
      core::slice::from_raw_parts_mut(self as *mut _ as usize as *mut u8, DIRENT_SZ) 
    }
  }

  /// Get name of the entry
  pub fn name(&self) -> &str {
    let len = (0usize..).find(|i| self.name[*i] == 0).unwrap();
    core::str::from_utf8(&self.name[..len]).unwrap()
  }

  /// Get inode number of the entry
  pub fn inode_number(&self) -> u32 {
    self.inode
  }
}

