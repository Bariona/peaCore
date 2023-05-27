#![no_std]

extern crate alloc;

mod block_dev;
mod block_cache;
mod bitmap;
mod layout;
mod fs;
mod vfs;

pub const BLOCK_SZ: usize = 512;
pub use fs::FileSystem;
pub use block_dev::BlockDevice;

type DataBlock = [u8; BLOCK_SZ];

