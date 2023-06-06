// use std::println;

use alloc::{sync::Arc, vec::Vec, string::{String, ToString}};
use spin::{Mutex, MutexGuard};

use crate::{fs::FileSystem, block_dev::BlockDevice, layout::{DiskInode, DIRENT_SZ, DirEntry, DiskInodeType}, block_cache::{get_block_cache, block_cache_sync_all}};


/// Different from `DiskInode`, `Inode` is stored in Memory
pub struct Inode {
  block_id: usize,            // corresponding block id
  block_offset: usize,        // inode's offset within the block
  fs: Arc<Mutex<FileSystem>>, // operations are achieved through `FileSystem`
  block_dev: Arc<dyn BlockDevice>,
}

impl Inode {

  /// create a vfs node
  pub fn new(
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<FileSystem>>,
    block_dev: Arc<dyn BlockDevice>,
  ) -> Self {
    Self {
      block_id,
      block_offset,
      fs,
      block_dev,
    }
  }

  fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
    get_block_cache(self.block_id, self.block_dev.clone())
      .lock()
      .read(self.block_offset, f)
  }

  fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
    get_block_cache(self.block_id, self.block_dev.clone())
      .lock()
      .modify(self.block_offset, f)
  }

  /// find inode by its name
  pub fn find_name(&self, name: &str) -> Option<Arc<Inode>> {
    let fs = self.fs.lock();
    self.read_disk_inode(|disk_inode| {
      self.find_inode_id(name, disk_inode).map(|inode_id| {
        let (block_id, inner_block_offset) = fs.get_disk_inode_pos(inode_id as usize);
        Arc::new(
          Self::new(
            block_id,
            inner_block_offset,
            self.fs.clone(),
            self.block_dev.clone(),
          )
        )
      })
    })
  }

  fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
    assert!(disk_inode.is_dir());
    let file_count = disk_inode.size as usize / DIRENT_SZ;

    assert!(file_count * DIRENT_SZ == disk_inode.size as usize, 
      "name = {}, {} {} {}", name, file_count, DIRENT_SZ, disk_inode.size);

    let mut dirent = DirEntry::empty();
    for i in 0..file_count {
      let buf_len = disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_dev);
      assert_eq!(buf_len, DIRENT_SZ);
      if dirent.name() == name {
        return Some(dirent.inode_number());
      }
    }
    None
  }

  /// Increase the size of disk inode
  fn increase_size(
    &self, 
    new_size: u32, 
    disk_inode: &mut DiskInode,
    fs: &mut MutexGuard<FileSystem>
  ) {
    if new_size < disk_inode.size {
      return;
    }
    let blocks_needed = disk_inode.blocks_num_needed(new_size);
    let mut new_blocks: Vec<u32> = Vec::new();
    for _ in 0..blocks_needed {
      new_blocks.push(fs.alloc_data());
    }
    disk_inode.increase_size(new_size, new_blocks, &self.block_dev);
  }

  /// Create Inode 
  pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
    let mut fs = self.fs.lock();
    let op = |root_inode: &DiskInode| {
      assert!(root_inode.is_dir());
      self.find_inode_id(name, root_inode)
    };
    if self.read_disk_inode(op).is_some() {
      return None;
    }
    let new_inode_id = fs.alloc_inode();
    let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id as usize);
    get_block_cache(block_id, self.block_dev.clone())
      .lock()
      .modify(block_offset, |new_inode: &mut DiskInode| {
        new_inode.initialize(DiskInodeType::File);
      });
    self.modify_disk_inode(|root_inode| {
      let file_count = (root_inode.size as usize) / DIRENT_SZ;
      assert_eq!(root_inode.size as usize, file_count * DIRENT_SZ);

      let new_size = (file_count + 1) * DIRENT_SZ;
      self.increase_size(new_size as u32, root_inode, &mut fs);
      let dirent = DirEntry::new(name, new_inode_id);
      root_inode.write_at(file_count * DIRENT_SZ, dirent.as_bytes(), &self.block_dev);
      // println!("{}", root_inode.size);
    });
    
    block_cache_sync_all();
    Some(Arc::new(
      Self::new(
        block_id, 
        block_offset, 
        self.fs.clone(), 
        self.block_dev.clone()
    )))
  }

  /// list inodes under current inode
  pub fn ls(&self) -> Vec<String> {
    let _fs = self.fs.lock();
    self.read_disk_inode(|disk_inode: &DiskInode| {
      assert!(disk_inode.is_dir());
      let file_count = disk_inode.size as usize / DIRENT_SZ;
      let mut file_name_list = Vec::new();
      for i in 0..file_count {
        let mut dirent = DirEntry::empty();
        let buf_len = disk_inode.read_at(
          i * DIRENT_SZ, 
          &mut dirent.as_bytes_mut(), 
          &self.block_dev
        );
        assert_eq!(buf_len, DIRENT_SZ);
        file_name_list.push(dirent.name().to_string());
      }
      file_name_list
    })
  }

  pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
    let _fs = self.fs.lock(); // lock file system (multi-core)
    self.read_disk_inode(|disk_inode: &DiskInode| {
      disk_inode.read_at(offset, buf, &self.block_dev)
    })
  }

  /// write `buf` to the file
  pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
    let mut fs = self.fs.lock();
    let buf_len = self.modify_disk_inode(|disk_inode: &mut DiskInode| {
      self.increase_size(disk_inode.size + buf.len() as u32, disk_inode, &mut fs);
      disk_inode.write_at(offset, buf, &self.block_dev)
    });
    block_cache_sync_all();
    buf_len
  }

  /// Clear the data in current inode but remains the inode
  pub fn clear(&self) {
    let mut fs = self.fs.lock();
    self.modify_disk_inode(|disk_inode: &mut DiskInode| {
      let tot_blks = DiskInode::total_blocks(disk_inode.size);
      let free_block_ids = disk_inode.clear_size(&self.block_dev);
      assert_eq!(tot_blks as usize, free_block_ids.len());
      for block_id in free_block_ids {
        fs.dealloc_data(block_id as usize);
      }
    });
    block_cache_sync_all();
  }
}

