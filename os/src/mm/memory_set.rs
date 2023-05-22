use core::arch::asm;
use core::cmp::max;

use alloc::{collections::BTreeMap, sync::Arc};

use bitflags::bitflags;
use alloc::vec::Vec;
use riscv::register::satp;
use crate::board::MMIO;
use crate::config::USER_STACK_TOP;
use crate::{config::{PAGE_SIZE, TRAMPOLINE, MEMORY_ENDPOINT, USER_STACK_SIZE, TRAP_CONTEXT}, mm::address::StepByOne, sync::up::UPSafeCell};

use super::{page_table::{PageTable, PTEFlags, PageTableEntry}, address::{VPNRange, VirtPageNum, VirtAddr, PhysPageNum, PhysAddr}, frame_allocator::{FrameTracker, frame_alloc}};

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

lazy_static! {
  pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = 
    Arc::new( unsafe { UPSafeCell::new(MemorySet::new_kernel()) } );
}

/// All MapAreas shares the same page_table, but their PTE permissions differ.
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

  pub fn token(&self) -> usize {
    self.page_table.token()
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

  /// Assume that no conflicts, insert framed area
  pub fn insert_framed_area(
    &mut self, 
    start_va: VirtAddr,
    end_va: VirtAddr,
    perm: MapPermission,
  ) {
    self.push(MapArea::new(start_va, end_va, MapType::Framed, perm), None);
  }

  /// ReMove `MapArea` that starts with `start_vpn`
  pub fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
    if let Some((idx, area)) = self
      .areas
      .iter_mut()
      .enumerate()
      .find(|(_, area)| area.vpn_range.get_start() == start_vpn) {
        area.unmap(&mut self.page_table);
        self.areas.remove(idx);
      }
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

    println!("mapping {:>9} r-x section [{:#x}, {:#x})", ".text", stext as usize, etext as usize);
    memory_set.push(
      MapArea::new(
        (stext as usize).into(), 
        (etext as usize).into(), 
        MapType::Identical, 
        MapPermission::R | MapPermission::X
      ), 
     None
    );

    println!("mapping {:>9} r-- section [{:#x}, {:#x})", ".rodata", srodata as usize, erodata as usize);
    memory_set.push(
      MapArea::new(
        (srodata as usize).into(),
        (erodata as usize).into(),
        MapType::Identical,
        MapPermission::R,
      ),
      None,
    );

    println!("mapping {:>9} rw- section [{:#x}, {:#x})", ".data", sdata as usize, edata as usize);
    memory_set.push(
      MapArea::new(
        (sdata as usize).into(),
        (edata as usize).into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
      ),
      None,
    );

    println!("mapping {:>9} rw- section [{:#x}, {:#x})", ".data", sbss_with_stack as usize, ebss as usize);
    memory_set.push(
      MapArea::new(
        (sbss_with_stack as usize).into(),
        (ebss as usize).into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
      ),
      None,
    );

    println!("mapping PhyMemory rw- [{:#x}, {:#x})", ekernel as usize, MEMORY_ENDPOINT);
    // same as frame_allocator
    memory_set.push(
      MapArea::new(
        (ekernel as usize).into(),
        MEMORY_ENDPOINT.into(),
        MapType::Identical,
        MapPermission::R | MapPermission::W,
      ),
      None,
    );

    for &pair in MMIO {
      memory_set.push(
        MapArea::new(
          pair.0.into(),
          (pair.0 + pair.1).into(),
          MapType::Identical,
          MapPermission::R | MapPermission::W,
        ),
        None,
      );
      println!("mapping MMIO rw- [{:#x}, {:#x})", pair.0, pair.0 + pair.1);
    }
    println!("kernel memory space mapping done.");
    memory_set
  }

  /// In user address space: create trampoline, trapContext and userStack,
  /// 
  /// also returns user_stack_pointer and its entry_point
  pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize, usize) {
    let mut memory_set = Self::new_bare();
    memory_set.map_trampoline();
    let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
    let elf_header = elf.header;
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");

    let ph_count = elf_header.pt2.ph_count();
    let mut max_end_vpn = VirtPageNum(0);
    for i in 0..ph_count {
      let ph = elf.program_header(i).unwrap();
      if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
        let start_va = (ph.virtual_addr() as usize).into();
        let end_va = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
        let mut map_perm = MapPermission::U;
        let ph_flags = ph.flags();
        if ph_flags.is_read() {
          map_perm |= MapPermission::R;
        }
        if ph_flags.is_write() {
          map_perm |= MapPermission::W;
        }
        if ph_flags.is_execute() {
          map_perm |= MapPermission::X;
        } 
        let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
        max_end_vpn = map_area.vpn_range.get_end();
        // println!("elf: begin: {:?}, end: {:?}, {:?}", start_va, end_va, map_perm);
        memory_set.push(
          map_area, 
          Some(&elf.input[ph.offset() as usize .. (ph.offset() + ph.file_size()) as usize])
        );
      }
    }
    let max_end_va: VirtAddr = max_end_vpn.into();
    let mut user_stack_bottom: usize = max_end_va.into();
    user_stack_bottom += PAGE_SIZE; // add guard page

    // let user_stack_top: usize = user_stack_bottom + USER_STACK_SIZE;
    user_stack_bottom = max(user_stack_bottom, USER_STACK_TOP - USER_STACK_SIZE);
    let user_stack_top: usize = USER_STACK_TOP;

    memory_set.push(
      MapArea::new(
        user_stack_bottom.into(),
        user_stack_top.into(),
        MapType::Framed,
        MapPermission::R | MapPermission::W | MapPermission:: U,
      ),
      None
    );

    memory_set.push(
      MapArea::new(
        user_stack_top.into(),
        user_stack_top.into(),
        MapType::Framed,
        MapPermission::R | MapPermission::W | MapPermission::U,
      ),
      None,
    );

    memory_set.push(
      MapArea::new(
        TRAP_CONTEXT.into(), 
        TRAMPOLINE.into(), 
        MapType::Framed, 
        MapPermission::R | MapPermission::W
      ),
      None
    );
    // return (memoryset, user_stack_top, _start)
    (memory_set, user_stack_bottom, user_stack_top, elf.header.pt2.entry_point() as usize)
  }

  pub fn activate(&self) {
    let satp = self.page_table.token();
    unsafe {
      satp::write(satp);
      asm!("sfence.vma");
    }
  }

  /// copy a user space
  pub fn from_existed_user(user_space: &Self) -> Self {
    let mut memory_set = Self::new_bare();
    memory_set.map_trampoline();
    for area in user_space.areas.iter() {
      let new_area = MapArea::from_another(area);
      memory_set.push(new_area, None);
      for vpn in area.vpn_range {
        let src_ppn = user_space.translate(vpn).unwrap().ppn();
        let dst_ppn = memory_set.translate(vpn).unwrap().ppn();
        dst_ppn.get_bytes_array().copy_from_slice(src_ppn.get_bytes_array());
      }
    }
    memory_set
  }
  /// Remove all `MapArea`
  pub fn recycle_data_pages(&mut self) {
    self.areas.clear();
  }

  #[allow(unused)]
  pub fn check_valid(&self, va: VirtAddr) -> bool {
    self.page_table.check_valid(va.floor())
  }

  /// translate a virtual page number
  pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
    self.page_table.translate(vpn)
  }

  pub fn shrink_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
    if let Some(area) = self
      .areas
      .iter_mut()
      .find(|area| area.vpn_range.get_start() == start.floor()) {
      
      area.shrink_to(&mut self.page_table, new_end.ceil());
      true
    } else {
      false
    }
  }

  pub fn append_to(&mut self, start: VirtAddr, new_end: VirtAddr) -> bool {
    if let Some(area) = self
      .areas
      .iter_mut()
      .find(|area| area.vpn_range.get_start() == start.floor()) {
      area.append_to(&mut self.page_table, new_end.ceil());
      true
    } else {
      false
    }
  }

  /// expand process's sp -> [new_start.floor(), sp_end]
  pub fn expand_sp(&mut self, sp_end: VirtAddr, new_start: VirtAddr) -> bool {
    if let Some(area) = self
      .areas
      .iter_mut()
      .find(|area| area.vpn_range.get_end() == sp_end.ceil()) {
      // println!("{:?} {:?} {:?} {:?}", sp_end, new_start, area.vpn_range.get_start(), area.vpn_range.get_end());
      area.expand_sp(&mut self.page_table, new_start.floor());
      true
    } else {
      false
    }
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
  vpn_range: VPNRange,
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
      vpn_range: VPNRange::new(start_vpn, end_vpn),
      data_frames: BTreeMap::new(),
      map_type,
      map_perm
    }
  }

  /// Without copy data_frames
  pub fn from_another(another: &Self) -> Self {
    Self {
      vpn_range: VPNRange::new(another.vpn_range.get_start(), another.vpn_range.get_end()),
      data_frames: BTreeMap::new(),
      map_type: another.map_type,
      map_perm: another.map_perm,
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
    // println!("begin: {:?} {:?}", self.vpn_range.get_start(), self.vpn_range.get_end());
    for vpn in self.vpn_range {
      self.map_one(page_table, vpn);
    }
    // println!("end: ");
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
    for vpn in self.vpn_range {
      self.unmap_one(page_table, vpn);
    }
  }

  pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
    // Question: HOW can you simply visit the physical memory??
    // Answer: Because in S-mode, we are identical mapped.
    // Thus, we can copy process's data from vpn to their corresponding ppn at first
    assert_eq!(self.map_type, MapType::Framed);
    let mut start: usize = 0;
    let mut current_vpn = self.vpn_range.get_start();
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
      assert!(current_vpn <= self.vpn_range.get_end());
    }
  }

  #[allow(unused)]
  /// [start, end] -> [start, new_end] (new_end <= end)
  pub fn shrink_to(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
    for vpn in VPNRange::new(new_end, self.vpn_range.get_end()) {
      self.unmap_one(page_table, vpn);
    }
    self.vpn_range = VPNRange::new(self.vpn_range.get_start(), new_end);
  }

  #[allow(unused)]
  /// [start, end] -> [start, new_end] (end <= new_end)
  pub fn append_to(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
    for vpn in VPNRange::new(self.vpn_range.get_end(), new_end) {
      self.map_one(page_table, vpn);
    }
    self.vpn_range = VPNRange::new(self.vpn_range.get_start(), new_end);
  }

  pub fn expand_sp(&mut self, page_table: &mut PageTable, new_sp: VirtPageNum) {
    // println!("{:?} {:?}", new_sp, self.vpn_range.get_end());
    for vpn in VPNRange::new(new_sp, self.vpn_range.get_start()) {
      self.map_one(page_table, vpn)
    }
    self.vpn_range = VPNRange::new(new_sp, self.vpn_range.get_end());
  }
}

#[allow(unused)]
pub fn remap_test() {
  let mut kernel_space = KERNEL_SPACE.exclusive_access();
  let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
  let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
  let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
  assert!(!kernel_space
    .page_table
    .translate(mid_text.floor())
    .unwrap()
    .writable());
  assert!(kernel_space
    .page_table
    .translate(mid_text.floor())
    .unwrap()
    .executable());
  assert!(!kernel_space
    .page_table
    .translate(mid_rodata.floor())
    .unwrap()
    .writable());
  assert!(!kernel_space
    .page_table
    .translate(mid_data.floor())
    .unwrap()
    .executable());
  println!("remap test: \x1b[92m[passed!]\x1b[0m");
}
