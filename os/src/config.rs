// Constants used in peaCore
#![allow(dead_code)]
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const MEMORY_ENDPOINT: usize = 0x80800000;

pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_BITS; // 4k


pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
