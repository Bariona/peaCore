use core::mem::MaybeUninit;

use spin::Mutex;
use uart_16550::MmioSerialPort;

pub trait ConsoleTrait: Sync {
  /// put a char to the console
  fn put_char(&self, c: u8);

  /// put a string to the console
  fn put_str(&self, s: &str);

  /// get a char
  fn get_char(&self) -> usize;
}

pub (crate) struct Console;
pub static UART: Mutex<MaybeUninit<MmioSerialPort>> = Mutex::new(MaybeUninit::uninit());

impl Console {
  /// initialize UART
  pub fn console_init(addr: usize) { 
    *UART.lock() = MaybeUninit::new(unsafe { MmioSerialPort::new(addr) });
  }
}

impl ConsoleTrait for Console {
  #[inline]
  fn put_char(&self, c: u8) {
    unsafe { UART.lock().assume_init_mut() }.send(c);
  }

  #[inline]
  fn put_str(&self, s: &str) {
    let mut uart = UART.lock();
    let uart = unsafe { uart.assume_init_mut() };
    for c in s.bytes() {
      uart.send(c);
    }
  }

  #[inline]
  fn get_char(&self) -> usize {
    let mut uart = UART.lock();
    let uart = unsafe { uart.assume_init_mut() };
    uart.receive() as usize
  }
}
