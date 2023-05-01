use self::{process::{sys_exit, sys_yield, sys_get_time}, fs::{sys_write, sys_read}};

mod process;
mod fs;

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_SBRK: usize = 214;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
  match syscall_id {
    SYSCALL_EXIT => sys_exit(args[0] as i32),
    SYSCALL_YIELD => sys_yield(),
    SYSCALL_GET_TIME => sys_get_time(),
    // SYSCALL_SBRK => sys_sbrk(args[0] as i32),
    SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
    SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
    _ => panic!("Unsupported syscall: {:#x} ", syscall_id)
  }
}