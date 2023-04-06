use alloc::vec::Vec;
use bitflags::bitflags;

use alloc::vec;
use riscv::addr::{Address};

use super::{address::{PhysPageNum, VirtPageNum, VirtAddr, StepByOne}, frame_allocator::{FrameTracker, frame_alloc}};

bitflags! {
  pub struct PTEFlags: u8 {
    const V = 1 << 0; // valid 
    const R = 1 << 1; // read
    const W = 1 << 2; // write
    const X = 1 << 3; // execute
    const U = 1 << 4; // accessible to U-mode(U = 1)
    const G = 1 << 5; // global (exist in all address space)
    const A = 1 << 6; // indicates the virtual page has been read, written, or fetched from since the last time the A bit was cleared
    const D = 1 << 7; // dirty
  }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)] // arrange struct in C style
/// SV39 page table entry
pub struct PageTableEntry {
  pub bits: usize
}

impl PageTableEntry {
  pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
    Self { bits: ppn.0 << 10 | (flags.bits as usize) }
  }
  
  pub fn empty() -> Self {
    Self { bits: 0 }
  }

  pub fn ppn(&self) -> PhysPageNum {
    assert!((self.bits >> 10) < (1 << 44));
    (self.bits >> 10 & ((1usize << 44) - 1)).into()
  }

  pub fn flags(&self) -> PTEFlags {
    PTEFlags::from_bits(self.bits as u8).unwrap()
  }

  pub fn is_valid(&self) -> bool {
    (self.flags() & PTEFlags::V) != PTEFlags::empty()
  }

  pub fn readable(&self) -> bool {
    (self.flags() & PTEFlags::R) != PTEFlags::empty()
  }

  pub fn writable(&self) -> bool {
    (self.flags() & PTEFlags::W) != PTEFlags::empty()
  }

  pub fn executable(&self) -> bool {
    (self.flags() & PTEFlags::X) != PTEFlags::empty()
  }
}

/// page table's all physical frames
pub struct PageTable {
  root_ppn: PhysPageNum,
  frames: Vec<FrameTracker>
}

/// Assume that there won't be oom while creating PT
impl PageTable {
  /// use FRAME_ALLOCATOR to create a frame
  pub fn new() -> Self {
    let frame = frame_alloc().unwrap();
    PageTable { 
      root_ppn: frame.ppn, 
      frames: vec![frame]
    }
  }

  /// Temporarily used to get arguments from user space.
  pub fn from_token(satp: usize) -> Self {
    Self { 
      root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)), 
      frames: Vec::new() 
    }
  } 

  /// Returns `satp` 
  pub fn token(&self) -> usize {
    // `8` means enable page_table
    8usize << 60 | self.root_ppn.0
  }

  /// find PTE, if page table doesn't exists, then create one.
  fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
    let idxs = vpn.indexes();
    let mut base_ppn = self.root_ppn;
    let mut result: Option<&mut PageTableEntry> = None;
    for (i, &idx) in idxs.iter().enumerate() {
      let pte = &mut base_ppn.get_pte_array()[idx];
      if i == 2 {
        result = Some(pte);
        break;
      } 
      if !pte.is_valid() {
        let frame = frame_alloc().unwrap();
        *pte = PageTableEntry::new(frame.ppn, PTEFlags::V); // declare it as valid
        self.frames.push(frame);
      }
      base_ppn = pte.ppn();
    }
    result
  }

  /// find PTE, otherwise will return `None`
  fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
    let idxs = vpn.indexes();
    let mut base_ppn = self.root_ppn;
    let mut result: Option<&mut PageTableEntry> = None;
    for (i, &idx) in idxs.iter().enumerate() {
      let pte = &mut base_ppn.get_pte_array()[idx];
      if i == 2 {
        result = Some(pte);
        break;
      } 
      if !pte.is_valid() {
        return None
      }
      base_ppn = pte.ppn();
    }
    result
  }

  /// map a virtual page to a physical page
  pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
    let pte = self.find_pte_create(vpn).unwrap();
    assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
    *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
  }

  /// set a virtual page as invalid
  pub fn unmap(&mut self, vpn: VirtPageNum) {
    let pte = self.find_pte(vpn).unwrap();
    assert!(pte.is_valid(), "vpn {:?} is invalid before mapping", vpn);
    *pte = PageTableEntry::empty();
  }

  /// try to translate a virtual page into PTE
  pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
    self.find_pte(vpn).map(|pte| *pte)
  }
}


/// translate a pointer to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
  let page_table = PageTable::from_token(token); // get base page
  let mut start = ptr as usize;
  let end = start + len;
  let mut bytes = Vec::new();

  while start < end {
    let start_va = VirtAddr::from(start);
    let mut start_vpn = start_va.floor();
    let start_ppn = page_table.translate(start_vpn).unwrap().ppn();
    start_vpn.step();
    let mut end_va: VirtAddr = start_vpn.into();
    end_va = end_va.min(VirtAddr::from(end));
    if end_va.page_offset() != 0 {
      bytes.push(&mut start_ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
    } else {
      bytes.push(&mut start_ppn.get_bytes_array()[start_va.page_offset()..]);
    }
    start = end_va.into();
  }
  
  bytes
}