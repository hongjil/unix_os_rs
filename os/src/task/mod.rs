mod context;
mod switch;
mod task;

use crate::config;
use crate::loader::{get_app_base, get_num_app, KERNEL_STACK, USER_STACK};
use crate::sync::UPSafeCell;
use crate::trap;
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
    tasks: [task::TaskControlBlock; config::MAX_APP_NUM],
    // The current running task.
    cur_task: usize,
}
// The macro lazy_static would postpone the initialization until the first time
// variables are used.
lazy_static! {
    static ref TASK_MANAGER: TaskManager = {
        let mut tasks = [TaskControlBlock {
            ctx: TaskContext::zero_init(),
            status: TaskStatus::UnInit,
        }; config::MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.ctx = TaskContext::init(KERNEL_STACK[i].push_context(
                trap::TrapContext::app_init_context(get_app_base(i), USER_STACK[i].get_sp()),
            ) as *const trap::TrapContext as usize);
            task.status = TaskStatus::Ready;
        }
        TaskManager {
            num_task: get_num_app(),
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks: tasks,
                    cur_task: 0,
                })
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
        self.print_tasks();
        // We just create an empty TaskContext which would be written unused
        // registers in __switch.
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_ctx_ptr);
        }
        panic!("Unreachable in TaskManager::run_first_task")
    }
    /// Change the status of current `Running` task into `Ready`.
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
        task.status = TaskStatus::Ready;
    }
    /// Change the status of current `Running` task into `Exited`. Panic
    /// if the current task is not in Running status.
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
    fn run_next_task(&self) -> ! {
        if let Some(next_task_id) = self.find_next_task() {
            let mut inner = TASK_MANAGER.inner.exclusive_access();
            let current_task_id = inner.cur_task;
            let cur_task_ptr = &mut inner.tasks[current_task_id].ctx as *mut TaskContext;
            let next_task_ptr = &inner.tasks[next_task_id].ctx as *const TaskContext;
            inner.cur_task = next_task_id;
            inner.tasks[next_task_id].status = TaskStatus::Running;
            drop(inner);
            self.print_tasks();
            unsafe {
                __switch(cur_task_ptr, next_task_ptr);
            }
            panic!("Unreachable in TaskManager::run_next_task")
        } else {
            panic!("All tasks are exited.");
        }
    }
    fn print_tasks(&self) {
        println!("[kernel] ===== Printing tasks info Start =====");
        let inner = TASK_MANAGER.inner.exclusive_access();
        inner.tasks.iter().enumerate().for_each(|(i, task)| {
            if i < TASK_MANAGER.num_task {
                println!("Task {} with Status {:?}", i, task.status);
            }
        });
        println!("[kernel] ===== Printing tasks info Done  =====");
    }
}

/// Run the first task.
pub fn run_first_task() -> ! {
    TASK_MANAGER.run_first_task();
}

/// Suspends the current task, then run the next task.
pub fn suspend_current_and_run_next() -> ! {
    TASK_MANAGER.mark_current_suspended();
    TASK_MANAGER.run_next_task();
}

/// Exits the current task,  then run the next task.
pub fn exit_current_and_run_next() -> ! {
    TASK_MANAGER.mark_current_exited();
    TASK_MANAGER.run_next_task();
}
