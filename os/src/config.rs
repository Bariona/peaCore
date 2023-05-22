// Constants used in peaCore
pub const USER_STACK_TOP: usize = TRAP_CONTEXT - PAGE_SIZE;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const USER_STACK_MAX_SIZE: usize = 4096 * 128;

pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const MEMORY_ENDPOINT: usize = 0x81000000; // KERNEL_MAX_MEMORY_ALLOCATED

pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_BITS; // 4k


pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

pub use crate::board::CLOCK_FREQ;
