use alloc::vec::Vec;
use bitflags::bitflags;

use alloc::vec;

use super::{address::{PhysPageNum, VirtPageNum}, frame_allocator::{FrameTracker, frame_alloc}};

bitflags! {
  pub struct PTEFlags: u8 {
    const V = 1 << 0;
    const R = 1 << 1;
    const W = 1 << 2;
    const X = 1 << 3;
    const U = 1 << 4;
    const G = 1 << 5;
    const A = 1 << 6;
    const D = 1 << 7;
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
    let pte = self.find_pte_create(vpn).unwrap();
    assert!(pte.is_valid(), "vpn {:?} is invalid before mapping", vpn);
    *pte = PageTableEntry::empty();
  }

  /// try to translate a virtual page into PTE
  pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
    self.find_pte(vpn).map(|pte| *pte)
  }
}