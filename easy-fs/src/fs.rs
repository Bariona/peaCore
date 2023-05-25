use alloc::sync::Arc;
use spin::Mutex;

use crate::{block_dev::BlockDevice, bitmap::Bitmap, layout::{DiskInode, SuperBlock, DiskInodeType}, BLOCK_SZ, block_cache::{get_block_cache, block_cache_sync_all}, DataBlock};

pub struct FileSystem {
  pub block_dev: Arc<dyn BlockDevice>,
  pub inode_bitmap: Bitmap,
  pub data_bitmap: Bitmap,

  inode_area_start_block: u32,
  data_area_start_block: u32,
}

impl FileSystem {
  pub fn create(
    block_dev: Arc<dyn BlockDevice>,
    total_blks: u32,
    inode_bitmap_blks: u32,
  ) -> Arc<Mutex<Self>> {
    let inode_bitmap = Bitmap::new(1, inode_bitmap_blks as usize);
    let inode_num = inode_bitmap.maximum();  // number of total inodes
    let inode_area_blks = (inode_num * core::mem::size_of::<DiskInode>() + BLOCK_SZ - 1) / BLOCK_SZ;

    let data_tot_blks = total_blks - 1 - inode_bitmap_blks - inode_area_blks as u32;
    let data_bitmap_blks = (data_tot_blks as usize + (BLOCK_SZ * 8) - 1) / (BLOCK_SZ * 8);
    let data_area_blks = data_tot_blks as usize - data_bitmap_blks;
    let data_bitmap = Bitmap::new(1 + inode_bitmap_blks as usize + inode_area_blks, data_bitmap_blks);

    let mut fs = Self {
      block_dev: block_dev.clone(),
      inode_bitmap,
      data_bitmap,

      inode_area_start_block: 1 + inode_bitmap_blks,
      data_area_start_block: 1 + inode_bitmap_blks + inode_area_blks as u32 + data_bitmap_blks as u32,
    };

    // initialize with zero
    for i in 0..total_blks {
      get_block_cache(i as usize, block_dev.clone())
        .lock()
        .modify(0, |blk: &mut DataBlock|{
          blk.iter_mut().for_each(|byte| *byte = 0);
        }) 
    }

    get_block_cache(0, block_dev.clone()) 
      .lock()
      .modify(0, |super_blk: &mut SuperBlock| {
        *super_blk = SuperBlock::new(
          total_blks, 
          inode_bitmap_blks, 
          inode_area_blks as u32, 
          data_bitmap_blks as u32, 
          data_area_blks as u32
        );
      });

    assert_eq!(0, fs.alloc_inode());
    // initiliaze `/`
    let (root_inode_blk_id, root_inode_offset) = fs.get_disk_inode_pos(0);
    assert_eq!(0, root_inode_offset);
    get_block_cache(root_inode_blk_id, block_dev.clone())
      .lock()
      .modify(root_inode_offset, |root_inode: &mut DiskInode| {
        root_inode.initialize(DiskInodeType::Directory);
      });
    block_cache_sync_all();
    Arc::new(Mutex::new(fs))
  }

  /// Open a block device as a filesystem
  pub fn open(block_dev: Arc<dyn BlockDevice>) -> Arc<Mutex<Self>> {
    let fs = get_block_cache(0, block_dev.clone()) 
      .lock()
      .read(0, |super_blk: &SuperBlock| {
        Self {
          block_dev: block_dev.clone(),
          inode_bitmap: Bitmap::new(1, super_blk.inode_bitmap_blocks as usize),
          data_bitmap: Bitmap::new(
            1 + super_blk.inode_bitmap_blocks as usize + super_blk.inode_area_blocks as usize, 
            super_blk.data_bitmap_blocks as usize
          ),
          inode_area_start_block: 1 + super_blk.inode_bitmap_blocks,
          data_area_start_block: 1 + super_blk.inode_bitmap_blocks + super_blk.inode_area_blocks + super_blk.data_bitmap_blocks, 
        }
      });
    Arc::new(Mutex::new(fs))
  }

  pub fn alloc_inode(&mut self) -> u32 {
    self.inode_bitmap.alloc(&self.block_dev).unwrap() as u32
  }

  pub fn dealloc_inode(&mut self, inode_id: usize) {
    self.inode_bitmap.dealloc(&self.block_dev, inode_id)
  }

  /// available data_block's id in the device's layout
  pub fn alloc_data(&mut self) -> u32 {
    self.data_bitmap.alloc(&self.block_dev).unwrap() as u32 + self.data_area_start_block
  }

  pub fn dealloc_data(&mut self, data_id: usize) {
    get_block_cache(data_id, self.block_dev.clone())
      .lock()
      .modify(0, |data: &mut DataBlock| {
        data.iter_mut().for_each(|byte| *byte = 0);
      });
    self.data_bitmap.dealloc(&self.block_dev, data_id - self.data_area_start_block as usize);
  }

  /// returns (block_id, inner_block_offset)
  pub fn get_disk_inode_pos(&self, inode_id: usize) -> (usize, usize) {
    let inode_size = core::mem::size_of::<DiskInode>();
    let inodes_per_block = BLOCK_SZ / inode_size;
    let block_id = self.inode_area_start_block as usize + inode_id / inodes_per_block;
    (
      block_id,
      inode_id % inodes_per_block * inode_size
    )
  }
}
