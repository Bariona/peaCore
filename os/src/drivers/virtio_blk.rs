use alloc::vec::Vec;
use easy_fs::BlockDevice;

use virtio_drivers::{Hal, VirtIOBlk, VirtIOHeader};

use crate::{sync::UPSafeCell, mm::{address::{PhysPageNum, PhysAddr, StepByOne}, frame_alloc, frame_dealloc, PageTable, kernel_token, FrameTracker}, board::VIRTIO_BASE_ADDRESS};

/// a simple virtio block device
pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<'static, VirtioHal>>);

impl VirtIOBlock {
  pub fn new() -> Self {
    Self(unsafe {
      UPSafeCell::new(
        VirtIOBlk::<VirtioHal>::new(&mut *(VIRTIO_BASE_ADDRESS as *mut VirtIOHeader)).unwrap()
      )
    })
  }
}

impl BlockDevice for VirtIOBlock {
  fn read_block(&self, block_id: usize, buf: &mut [u8]) {
    self.0.exclusive_access().read_block(block_id, buf).expect("error at reading");
  }

  fn write_block(&self, block_id: usize, buf: &[u8]) {
    self.0.exclusive_access().write_block(block_id, buf).expect("error at writing");
  }
}


lazy_static! {
  /// to prevent frames being dealloced
  static ref QUEUE_FRAMES: UPSafeCell<Vec<FrameTracker>> = unsafe { UPSafeCell::new(Vec::new()) };
}

pub struct VirtioHal;

impl Hal for VirtioHal {
  fn dma_alloc(pages: usize) -> virtio_drivers::PhysAddr {
    let mut ppn_base: PhysPageNum = PhysPageNum(19260817);
    for i in 0..pages {
      let frame = frame_alloc().unwrap();
      if i == 0 {
        ppn_base = frame.ppn;
      }
      assert_eq!(frame.ppn.0, ppn_base.0 + i, "i = {}", i);
      QUEUE_FRAMES.exclusive_access().push(frame);
    }
    let pa: PhysAddr = ppn_base.into();
    pa.0
  }

  fn dma_dealloc(paddr: virtio_drivers::PhysAddr, pages: usize) -> i32 {
    let pa = PhysAddr::from(paddr);
    let mut ppn: PhysPageNum = pa.into();    
    for _ in 0..pages {
      frame_dealloc(ppn);
      ppn.step();
    }
    0
  }

  fn phys_to_virt(paddr: virtio_drivers::PhysAddr) -> virtio_drivers::VirtAddr {
    paddr
  }

  fn virt_to_phys(vaddr: virtio_drivers::VirtAddr) -> virtio_drivers::PhysAddr {
    // might be data from kernel_stack, which is not identically mapped
    let phyaddr = PageTable::from_token(kernel_token())
      .translate_va(vaddr.into())
      .unwrap().0;
    phyaddr
  }
}