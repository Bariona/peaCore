use core::{cell::RefMut, mem};

use alloc::{rc::Weak, vec::Vec, sync::Arc};

use crate::{mm::{memory_set::{MemorySet, KERNEL_SPACE, MapPermission, self}, address::{PhysPageNum, VirtAddr}}, config::TRAP_CONTEXT, trap::{kernel_stack_position, context::TrapContext, trap_handler}, sync::up::UPSafeCell};

use super::{context::TaskContext, pid::{PidHandler, KernelStack, pid_alloc}};

#[derive(PartialEq, Clone, Copy)]
pub enum TaskStatus {
  Ready,
  Running,
  Zombie
}

pub struct TaskControlBlock {
  pub pid: PidHandler,
  pub kernel_stack: KernelStack,
  inner: UPSafeCell<TaskControlBlockInner>
}

pub struct TaskControlBlockInner {
  pub task_status: TaskStatus,
  pub memory_set: MemorySet,    /// task's user memory space
  pub task_cx: TaskContext,
  pub trap_cx_ppn: PhysPageNum, /// trap function() (i.e. trampoline)'s physical page number
  pub base_size: usize,         /// total memory size

  pub parent: Option<Weak<TaskControlBlock>>,
  pub children: Vec<Arc<TaskControlBlock>>,
  pub exit_code: i32  
}


impl TaskControlBlockInner {
  pub fn get_trap_cx(&self) -> &'static mut TrapContext {
    self.trap_cx_ppn.get_mut()
  }
  pub fn get_user_token(&self) -> usize {
    self.memory_set.token()
  }
  pub fn get_status(&self) -> TaskStatus {
    self.task_status
  }
  pub fn is_zombie(&self) -> bool {
    self.task_status == TaskStatus::Zombie
  }
}

impl TaskControlBlock {
  pub fn inner_exclusive_access(&self) -> RefMut<TaskControlBlockInner> {
    self.inner.exclusive_access()
  }

  pub fn new(elf_data: &[u8]) -> Self {
    let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    // note that `memory_set` is for the user address space
    // but, ... we are currently in the S-mode
    let trap_cx_ppn = memory_set
      .translate(VirtAddr::from(TRAP_CONTEXT).into())
      .unwrap()
      .ppn();
    let pid_handler = pid_alloc();
    let kernel_stack = KernelStack::new(pid_handler); // allocate process's kernel stack
    let kernel_stack_top = kernel_stack.get_top();

    let task_control_block = TaskControlBlock {
        pid: pid_handler,
        kernel_stack,
        inner: unsafe {
          UPSafeCell::new(TaskControlBlockInner {
            task_status: TaskStatus::Ready,
            memory_set,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            trap_cx_ppn,
            base_size: user_sp,
            parent: None,
            children: Vec::new(),
            exit_code: 0,
        })},
    };
    let trap_cx = task_control_block.get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
      entry_point, 
      user_sp, 
      KERNEL_SPACE.exclusive_access().token(), 
      kernel_stack_top, 
      trap_handler as usize
    );
    task_control_block
  }
  
  pub fn getpid(&self) -> usize {
    self.pid.0
  }

  pub fn exec(&self, elf_data: &[u8]) {
    let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    let trap_cx_ppn = memory_set
      .translate(VirtAddr::from(TRAP_CONTEXT).into())
      .unwrap()
      .ppn();
  
    let mut inner = self.inner_exclusive_access();
    // update PCB's info
    inner.memory_set = memory_set;
    inner.trap_cx_ppn = trap_cx_ppn;
    inner.base_size = user_sp;
    let trap_cx = inner.get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
      entry_point, 
      user_sp, 
      KERNEL_SPACE.exclusive_access().token(),
      self.kernel_stack.get_top(), 
      trap_handler
    );
    // === release inner automaticallly ===
  }

  pub fn fork(self: &Arc<Self>) -> Arc<Self> { 
    let mut parent_inner = self.inner.exclusive_access();
    
    let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
    let trap_cx_ppn = memory_set
      .translate(VirtAddr::from(TRAP_CONTEXT).into())
      .unwrap()
      .ppn();
  
    let pid_handler = pid_alloc();
    let kernel_stack = KernelStack::new(pid_handler);
    let kernel_stack_top = kernel_stack.get_top();

    let task_control_block = Arc::new(TaskControlBlock {
      pid: pid_handler,
      kernel_stack,
      inner: unsafe {
        UPSafeCell::new(TaskControlBlockInner {
          task_status: parent_inner.task_status, // TODO: task_status: Ready
          memory_set,
          task_cx: TaskContext::goto_trap_return(kernel_stack_top),
          trap_cx_ppn,
          base_size: parent_inner.base_size,
          parent: Some(Arc::downgrade(self)),
          children: Vec::new(),
          exit_code: 0,
      })},
    });

    parent_inner.children.push(task_control_block.clone());
    let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
    trap_cx.kernel_sp = kernel_stack_top;

    task_control_block
  } 
  
}