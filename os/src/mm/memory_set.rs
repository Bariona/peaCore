use alloc::collections::BTreeMap;

use bitflags::bitflags;
use alloc::vec::Vec;
use crate::{config::{PAGE_SIZE, TRAMPOLINE, MEMORY_ENDPOINT}, mm::address::StepByOne};

use super::{page_table::{PageTable, self, PTEFlags}, address::{VPNRange, VirtPageNum, VirtAddr, PhysPageNum, PhysAddr}, frame_allocator::{FrameTracker, frame_alloc}};

extern "C" {
  fn stext();
  fn etext();
  fn srodata();
  fn erodata();
  fn sdata();
  fn edata();
  fn sbss_with_stack();
  fn ebss();
  fn ekernel();
  fn strampoline();
}

pub struct MemorySet {
  page_table: PageTable,
  areas: Vec<MapArea>
}

impl MemorySet {
  pub fn new_bare() -> Self {
    Self { 
      page_table: PageTable::new(), 
      areas: Vec::new() 
    }
  }    

  /// Push map_area into a MemorySet 
  /// If data != None, also write data into map_area
  pub fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
    map_area.map(&mut self.page_table);
    if let Some(data) = data {
      map_area.copy_data(&mut self.page_table, data);
    }
    self.areas.push(map_area);
  }

  pub fn insert_framed_area(
    &mut self, 
    start_va: VirtAddr,
    end_va: VirtAddr,
    perm: MapPermission,
  ) {
    self.push(MapArea::new(start_va, end_va, MapType::Framed, perm), None);
  }

  /// trampoline's virtual address (256GB - 4k, 256GB]
  fn map_trampoline(&mut self) {
    self.page_table.map(
      VirtAddr::from(TRAMPOLINE).into(), 
      PhysAddr::from(strampoline as usize).into(), 
      PTEFlags::R | PTEFlags::X
    );
  }

  pub fn new_kernel() -> Self {
    let mut memory_set: MemorySet = Self::new_bare();
    memory_set.map_trampoline();
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(".bss [{:#x}, {:#x})",  sbss_with_stack as usize, ebss as usize);

    println!("mapping .text section");
    memory_set.push(
      MapArea::new(
        (stext as usize).into(), 
        (etext as usize).into(), 
        MapType::Identical, 
        MapPermission::R | MapPermission::X
      ), 
     None
    );

    println!("mapping .rodata section");
    memory_set.push(
      MapArea::new(
        (srodata as usize).into(),
        (erodata as usize).into(),
        MapType::Identical,
        MapPermission::R,
      ),
      None,
    );

    println!("mapping .data section");
    memory_set.push(
      MapArea::new(
        (sdata as usize).into(),
        (edata as usize).into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
      ),
      None,
    );

    println!("mapping .bss section");
    memory_set.push(
      MapArea::new(
        (sbss_with_stack as usize).into(),
        (ebss as usize).into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
      ),
      None,
    );

    println!("mapping physical memory");
    memory_set.push(
      MapArea::new(
        (ekernel as usize).into(),
        MEMORY_ENDPOINT.into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
      ),
      None,
    );
    memory_set
  }

}

#[derive(Debug, Clone, Copy, PartialEq)]
/// map type for memory set: identical or framed
pub enum MapType {
  Identical,
  Framed
}

bitflags! {
  /// map permission corresponding to that in pte: `R W X U`
  pub struct MapPermission: u8 {
    const R = 1 << 1;
    const W = 1 << 2;
    const X = 1 << 3;
    const U = 1 << 4;
  }
}

/// `Logical Section`: bunches of virtual page <-> physical page
/// and with their permission
pub struct MapArea {
  vpn_page: VPNRange,
  data_frames: BTreeMap<VirtPageNum, FrameTracker>,
  map_type: MapType,
  map_perm: MapPermission
}

impl MapArea {
  pub fn new(
    start_va: VirtAddr,
    end_va: VirtAddr,
    map_type: MapType,
    map_perm: MapPermission
  ) -> Self {
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();
    Self {
      vpn_page: VPNRange::new(start_vpn, end_vpn),
      data_frames: BTreeMap::new(),
      map_type,
      map_perm
    }
  }

  pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
    let ppn: PhysPageNum;
    match self.map_type {
      MapType::Identical => {
        ppn = PhysPageNum(vpn.0)
      } 
      MapType::Framed => {
        let frame = frame_alloc().unwrap();
        ppn = frame.ppn;
        self.data_frames.insert(vpn, frame);
      }
    }
    let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
    page_table.map(vpn, ppn, pte_flags);
  }
  pub fn map(&mut self, page_table: &mut PageTable) {
    for vpn in self.vpn_page {
      self.map_one(page_table, vpn);
    }
  }

  pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
    match self.map_type {
      MapType::Framed => {
        self.data_frames.remove(&vpn);
      }
      _ => {}
    }
    page_table.unmap(vpn);
  }
  pub fn unmap(&mut self, page_table: &mut PageTable) {
    for vpn in self.vpn_page {
      self.unmap_one(page_table, vpn);
    }
  }

  pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
    // TODO: 如何做到直接访问物理内存的??
    assert_eq!(self.map_type, MapType::Framed);
    let mut start: usize = 0;
    let mut current_vpn = self.vpn_page.get_start();
    let len = data.len();
    loop {
      let src = &data[start..len.min(start + PAGE_SIZE)];
      let dst = &mut &mut page_table
        .translate(current_vpn).unwrap()
        .ppn()
        .get_bytes_array()[..src.len()];
      dst.copy_from_slice(src);
      start += PAGE_SIZE;
      if start >= len {
        break;
      }
      current_vpn.step();
      assert!(current_vpn <= self.vpn_page.get_end());
    }
  }
}
