mod context;
mod switch;
mod task;

use crate::loader::{get_app_data, get_num_app};
use crate::mm::MemorySet;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use alloc::vec::Vec;
use context::TaskContext;
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

struct TaskManager {
    // The number of tasks.
    num_task: usize,
    // The state of each task and current running task.
    // Putting into under a wrapper since it's mutable.
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    // The control block of each task.
    tasks: Vec<TaskControlBlock>,
    // The current running task.
    cur_task: usize,
}
// The macro lazy_static would postpone the initialization until the first time
// variables are used.
lazy_static! {
    static ref TASK_MANAGER: TaskManager = {
        println!("[kernel] Initializing task manager");
        let num_app = get_num_app();
        println!("[kernel] Num of applications: {}", num_app);

        let mut tasks: Vec<TaskControlBlock> = Vec::new();

        for i in 0..num_app {
            tasks.push(match TaskControlBlock::new(get_app_data(i), i) {
                Ok(tcb) => tcb,
                Err(err) => {
                    panic!("Failed initialize control block for app {} with error: {:?}", i, err)
                }
            })
        }
        debug!("Initializing app/task done!");
        TaskManager {
            num_task: num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner { tasks, cur_task: 0 })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = TASK_MANAGER.inner.exclusive_access();
        inner.tasks[0].status = TaskStatus::Running;
        let next_task_ctx_ptr = &inner.tasks[0].ctx as *const TaskContext;
        drop(inner);
        // We just create an empty TaskContext which would be written unused
        // registers in __switch. Since it's not registered in the TaskManager,
        // it would never switch back to this fake TaskContext and never return.
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_ctx_ptr);
        }
        panic!("Unreachable in TaskManager::run_first_task")
    }

    // Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = TASK_MANAGER.inner.exclusive_access();
        let current_task_id = inner.cur_task;
        let task = &mut inner.tasks[current_task_id];
        if task.status != TaskStatus::Running {
            panic!(
                "Suspending a non-Running task {} with status {:?}",
                current_task_id, task.status
            );
        }

        debug!("Suspending the running task {}", current_task_id);

        task.status = TaskStatus::Ready;
    }

    // Change the status of current `Running` task into `Exited`. Panic
    // if the current task is not in Running status.
    fn mark_current_exited(&self) {
        let mut inner = TASK_MANAGER.inner.exclusive_access();
        let current_task_id = inner.cur_task;
        let task = &mut inner.tasks[current_task_id];
        if task.status != TaskStatus::Running {
            panic!(
                "Exiting a non-Running task {} with status {:?}",
                current_task_id, task.status
            );
        }
        println!("[kernel] Exiting the running task {}", current_task_id);
        task.status = TaskStatus::Exited;
    }
    fn find_next_task(&self) -> Option<usize> {
        let inner = TASK_MANAGER.inner.exclusive_access();
        for i in 1..=self.num_task {
            let next_task_id = (inner.cur_task + i) % self.num_task;
            if inner.tasks[next_task_id].status == TaskStatus::Ready {
                return Some(next_task_id);
            }
        }
        None
    }
    fn run_next_task(&self) {
        if let Some(next_task_id) = self.find_next_task() {
            let mut inner = TASK_MANAGER.inner.exclusive_access();
            let current_task_id = inner.cur_task;
            let cur_task_ptr =
                &mut inner.tasks[current_task_id].ctx as *mut TaskContext;
            let next_task_ptr =
                &inner.tasks[next_task_id].ctx as *const TaskContext;
            inner.cur_task = next_task_id;
            inner.tasks[next_task_id].status = TaskStatus::Running;
            drop(inner);

            debug!(
                "switching task from {} to {}",
                current_task_id, next_task_id
            );

            unsafe {
                __switch(cur_task_ptr, next_task_ptr);
            }
        } else {
            panic!("All tasks are exited normally.");
        }
    }

    // Returns physical page number of page table in the current task context.
    fn get_current_memory_set(&self) -> Arc<UPSafeCell<MemorySet>> {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.cur_task].memory_set.clone()
    }

    // Returns trap context in the current task context.
    fn get_current_trap_ctx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.cur_task].trap_ctx_ppn.get_mut()
    }

    // Returns current task id.
    fn get_current_idx(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.cur_task
    }
}

/// Run the first task.
/// This function never returns since run_first_task never returns.
pub fn run_first_task() -> ! {
    TASK_MANAGER.run_first_task()
}

/// Suspends the current task, then run the next task.
/// Other than the other function, this does return since when we switched back
/// it needs to continue to run.
pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

/// Exits the current task, then run the next task.
/// This function never returns since we never switched back to an exited task.
pub fn exit_current_and_run_next() -> ! {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
    panic!("Unreachable in exit_current_and_run_next()");
}

/// Return the address of root page table for the current task.
pub fn current_user_memory_set() -> Arc<UPSafeCell<MemorySet>> {
    TASK_MANAGER.get_current_memory_set()
}

/// Return the address of TrapContext for the current task.
pub fn current_trap_ctx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_ctx()
}

/// Return the current running app idx.
pub fn current_idx() -> usize {
    TASK_MANAGER.get_current_idx()
}
