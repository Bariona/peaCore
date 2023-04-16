use core::arch::{asm, global_asm};

use csr_riscv::register::{mie, mepc, mstatus::{self, MPP}, mtvec, utvec::TrapMode, mcause, sie, mscratch};

use crate::{board::{QEMU_BASE_ADDRESS, KERNEL_MAX_ALLOCED_ADDRESS, UART_BASE_ADDRESS}, rust_main, uart::Console, config::CLOCK_FREQ};

const CLINT: usize = 0x2000000;
const MTIMER_OFFSET: usize = 0x4000;
const MTIME_OFFSET: usize = 0xBFF8;


#[no_mangle]
#[repr(align(4))]
pub fn strap_handler() {
  println!("trap from S-mode");
  println!("cause: {:?} mepc: {:#x}", mcause::read().cause(), mepc::read());
  loop {
  }
}

#[inline(always)]
fn hart_id() -> usize {
  riscv::register::mhartid::read()
}

#[no_mangle]
pub fn start() {
  // set M Exception Program Counter to main, for mret.
  unsafe { 
    mstatus::set_mpp(MPP::Supervisor); 
    mtvec::write(strap_handler as usize, TrapMode::Direct);
  }
  mepc::write(rust_main as usize);
  unsafe {
    asm!("csrw mideleg,    {}", in(reg) !0);
    asm!("csrw medeleg,    {}", in(reg) !0);
  }
  unsafe {
    sie::set_ssoft();   // SSIE
    sie::set_stimer();  // STIE
    sie::set_sext();    // SEIE
  }

  timer_init();
  set_pmp();
	Console::console_init(UART_BASE_ADDRESS);
  
  println!("hart id = {}", hart_id());

  unsafe { asm!("mret"); }
}


fn clint_mtimecmp(id: usize) -> usize {
  CLINT + MTIMER_OFFSET + 8 * id
}

fn clint_mtime() -> usize {
  CLINT + MTIME_OFFSET
}

static mut SCRATCH: [usize; 5] = [0; 5];

global_asm!(include_str!("timervec.S"));

fn timer_init() {
  extern "C" {
    fn timervec();
  }
  let id = hart_id();
  let interval = CLOCK_FREQ / 10;
  unsafe {
    *(clint_mtimecmp(id) as *mut usize) = *(clint_mtime() as *const usize) + interval;
  }
  
  unsafe {
    SCRATCH[3] = clint_mtimecmp(id);
    SCRATCH[4] = interval;
    mscratch::write(&mut SCRATCH as *mut [usize; 5] as usize);

    // set the machine-mode trap handler.
    mtvec::write(timervec as usize, TrapMode::Direct);

    mstatus::set_mie(); 
    mie::set_mtimer();
  }
}

/// set PMP in M-mode
fn set_pmp() {
	use csr_riscv::register::*;
	unsafe {
		pmpcfg0::set_pmp(0, Range::OFF, Permission::NONE, false); // null pointer dereference
		pmpaddr0::write(0);
		// peripherals
		pmpcfg0::set_pmp(1, Range::TOR, Permission::RW, false);
		pmpaddr1::write(QEMU_BASE_ADDRESS >> 2);
		// kernel
		pmpcfg0::set_pmp(2, Range::TOR, Permission::RWX, false);
		pmpaddr2::write(KERNEL_MAX_ALLOCED_ADDRESS >> 2);
		// others
		pmpcfg0::set_pmp(3, Range::TOR, Permission::RW, false);
		pmpaddr3::write(1 << (usize :: BITS - 1));
	}
}