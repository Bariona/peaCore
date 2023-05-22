use core::arch::global_asm;
use core::arch::asm;

use riscv::register::{utvec::TrapMode, stvec, scause, stval, scause::{Trap, Exception, Interrupt}};

use crate::config::PAGE_SIZE;
use crate::config::USER_STACK_MAX_SIZE;
use crate::config::USER_STACK_TOP;
use crate::mm::address::VirtAddr;
use crate::syscall::syscall;
use crate::task::processor::current_task;
use crate::task::processor::current_trap_cx;
use crate::task::processor::current_user_token;
use crate::{config::{TRAP_CONTEXT, TRAMPOLINE}, task::{exit_current_and_run_next, suspend_current_and_run_next}};

pub mod context;

global_asm!(include_str!("trap.S"));

pub fn init() {
  set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
  unsafe {
    stvec::write(trap_from_kernel as usize, TrapMode::Direct);
  }
}

fn set_user_trap_entry() {
  unsafe {
    stvec::write(TRAMPOLINE, TrapMode::Direct);
  }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
  use riscv::register::mcause;
  let cause = mcause::read();
  println!("{:?}", cause);
  panic!("kernel's trap to do...")
}

#[no_mangle]
pub fn trap_handler() -> ! {
  set_kernel_trap_entry(); 
  let scause = scause::read();
  let stval = stval::read();
  match scause.cause() {
    Trap::Exception(Exception::UserEnvCall) => {
      let mut cx = current_trap_cx();
      cx.sepc += 4;
      let syscall_id = cx.x[17];
      // println!("handling syscall: {}", syscall_id);
      let result = syscall(syscall_id, [cx.x[10], cx.x[11], cx.x[12]]);
      cx = current_trap_cx();
      // println!("syscall: {} result: {}, current pid: {}", syscall_id, result, current_task().unwrap().getpid());
      cx.x[10] = result as usize;
    }
    Trap::Exception(Exception::StoreFault)
      | Trap::Exception(Exception::StorePageFault)
      | Trap::Exception(Exception::LoadFault)
      | Trap::Exception(Exception::LoadPageFault) => {
      let task = current_task().unwrap();
      let mut inner = task.inner_exclusive_access();
      let sp = inner.user_stack_bottom;
      if VirtAddr::from(stval).ceil() == sp && stval > USER_STACK_TOP - USER_STACK_MAX_SIZE {
        let new_sp: usize = Into::<usize>::into(VirtAddr::from(sp)) - 2 * PAGE_SIZE;
        inner.memory_set.expand_sp(USER_STACK_TOP.into(), VirtAddr::from(new_sp));
        inner.user_stack_bottom = VirtAddr::from(new_sp).into();
        drop(inner);
        drop(task);
        suspend_current_and_run_next();
      } else {
        drop(inner);
        drop(task);
        let cx = current_trap_cx();
        println!("[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", scause.cause(), stval, cx.sepc);
        exit_current_and_run_next(-2);
      }
    }
    Trap::Exception(Exception::IllegalInstruction) => {
      println!("[kernel] IllegalInstruction in application, kernel killed it.");
      exit_current_and_run_next(-3);
    }
    Trap::Interrupt(Interrupt::SupervisorTimer) => {
      panic!("timer interrupt is not implemented this way!");
    }
    Trap::Interrupt(Interrupt::SupervisorSoft) => {
      use csr_riscv::register::sip;
      unsafe { asm!("csrw sip,    {}", in(reg)sip::read().bits() & !2); } // clear SSIP: soft interruption pending bit
      suspend_current_and_run_next();
    }
    _ => {
      panic!(
        "Unsupported trap {:?}, stval = {:#x}!",
        scause.cause(),
        stval
      );
    }
  }
  trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
  // set user trap entry so that next time a trap happens, 
  // stvec will point to the trampoline.
  set_user_trap_entry();
  let trap_cx_ptr = TRAP_CONTEXT;
  let user_satp = current_user_token();
  // println!("{:?}", sstatus::read().spie());
  extern "C" {
    fn __alltraps();
    fn __restore();
  }
  let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
  unsafe {
    asm!(
      "fence.i",                      // clear the cache
      "jr {restore_va}",              // jump to new addr of __restore asm function
      restore_va = in(reg) restore_va,
      in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
      in("a1") user_satp,        // a1 = phy addr of usr page table
      options(noreturn)
    );
  }
}