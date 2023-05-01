//!Implementation of [`TaskManager`]

use alloc::{vec::Vec, sync::Arc, collections::VecDeque};

use crate::sync::up::UPSafeCell;

use super::{task::TaskControlBlock, context::TaskContext};

pub struct TaskManager {
  ready_queue: VecDeque<Arc<TaskControlBlock>>
}

/// Current Schedule Strategy: Round-Robin
impl TaskManager {
  pub fn new() -> Self {
    Self { ready_queue: VecDeque::new() }
  }

  /// Add a task to `TaskManager`
  pub fn add(&mut self, task: Arc<TaskControlBlock>) {
    self.ready_queue.push_back(task);
  }


  ///Remove the first task and return it, or `None if `TaskManager` is empty 
  pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
    self.ready_queue.pop_front()
  }
}

lazy_static! {
  pub static ref TASK_MANAGER: UPSafeCell<TaskManager> = unsafe {
    UPSafeCell::new(TaskManager::new())
  };
}

/// add task to TASK_MANAGER
pub fn add_task(task: Arc<TaskControlBlock>) {
  TASK_MANAGER.exclusive_access().add(task)
}

/// fetch task from TASK_MANAGET
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
  TASK_MANAGER.exclusive_access().fetch()
}