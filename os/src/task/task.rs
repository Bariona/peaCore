use core::{cell::RefMut};

use alloc::{vec::Vec, vec, sync::{Arc, Weak}};

use crate::{mm::{memory_set::{MemorySet, KERNEL_SPACE}, address::{VirtAddr, PhysPageNum, VirtPageNum}}, config::TRAP_CONTEXT, trap::{context::TrapContext, trap_handler}, sync::up::UPSafeCell, fs::{File, Stdin, Stdout}};

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
  pub user_stack_bottom: VirtPageNum,
  pub task_cx: TaskContext,
  pub trap_cx_ppn: PhysPageNum, /// trap function() (i.e. trampoline)'s physical page number
  pub base_size: usize,         /// total memory size

  /// fd table, None: closed fd
  pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,

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
    self.get_status() == TaskStatus::Zombie
  }

  pub fn alloc_fd(&mut self) -> usize {
    for (i, fd) in self.fd_table.iter().enumerate() {
      if fd.is_none() {
        return i
      }
    }
    self.fd_table.push(None);
    self.fd_table.len() - 1
  }
}

impl TaskControlBlock {
  pub fn inner_exclusive_access(&self) -> RefMut<TaskControlBlockInner> {
    self.inner.exclusive_access()
  }

  pub fn new(elf_data: &[u8]) -> Self {
    let (memory_set, user_sp_bottom, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    // note that `memory_set` is for the user address space
    // but, ... we are currently in the S-mode
    let trap_cx_ppn = memory_set
      .translate(VirtAddr::from(TRAP_CONTEXT).into())
      .unwrap()
      .ppn();
    let pid_handler = pid_alloc();
    let kernel_stack = KernelStack::new(&pid_handler); // allocate process's kernel stack
    let kernel_stack_top = kernel_stack.get_top();

    let task_control_block = TaskControlBlock {
        pid: pid_handler,
        kernel_stack,
        inner: unsafe {
          UPSafeCell::new(TaskControlBlockInner {
            task_status: TaskStatus::Ready,
            memory_set,
            user_stack_bottom: user_sp_bottom.into(),
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            trap_cx_ppn,
            base_size: user_sp,
            fd_table: vec![
              // fd 0
              Some(Arc::new(Stdin)), 
              // fd 1
              Some(Arc::new(Stdout)), 
              // fd 2
              Some(Arc::new(Stdout))
            ],
            parent: None,
            children: Vec::new(),
            exit_code: 0,
        })},
    };
    let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
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
    let (memory_set, user_sp_bottom, user_sp, entry_point) = MemorySet::from_elf(elf_data);
    let trap_cx_ppn = memory_set
      .translate(VirtAddr::from(TRAP_CONTEXT).into())
      .unwrap()
      .ppn();
  
    let mut inner = self.inner_exclusive_access();
    // update PCB's info
    inner.memory_set = memory_set;
    inner.user_stack_bottom = VirtAddr::from(user_sp_bottom).floor();

    inner.trap_cx_ppn = trap_cx_ppn;
    inner.base_size = user_sp;
    let trap_cx = inner.get_trap_cx();
    *trap_cx = TrapContext::app_init_context(
      entry_point, 
      user_sp, 
      KERNEL_SPACE.exclusive_access().token(),
      self.kernel_stack.get_top(), 
      trap_handler as usize
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
    let kernel_stack = KernelStack::new(&pid_handler);
    let kernel_stack_top = kernel_stack.get_top();

    let mut fd_copy = Vec::new();
    for file in &parent_inner.fd_table {
      if let Some(inode) = file {
        fd_copy.push(Some(inode.clone()));
      } else {
        fd_copy.push(None);
      }
    }
    let task_control_block = Arc::new(TaskControlBlock {
      pid: pid_handler,
      kernel_stack,
      inner: unsafe {
        UPSafeCell::new(TaskControlBlockInner {
          task_status: parent_inner.task_status, // TODO: task_status: Ready
          memory_set,
          user_stack_bottom: parent_inner.user_stack_bottom,
          task_cx: TaskContext::goto_trap_return(kernel_stack_top),
          trap_cx_ppn,
          base_size: parent_inner.base_size,
          fd_table: fd_copy,
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