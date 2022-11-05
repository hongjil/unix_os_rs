use super::TaskContext;

use crate::mm::*;
use crate::trap::{trap_handler, TrapContext};

use crate::config::{kernel_stack_position, TRAP_CONTEXT_ADDR};

pub struct TaskControlBlock {
    pub ctx: TaskContext,
    pub status: TaskStatus,
    pub memory_set: MemorySet,
    // The physical address of Trap context.
    pub trap_ctx_ppn: PhysPageNum,
    pub base_size: usize,
}

/* The state transition of a task:

                 +-------+
    +----------->| Ready |       +--------+
    | Init       +-+-----+       | Exited |
+---+----+ run_as_ |   ^         +--------+
| UnInit |  next   v   | yield       ^
+--------+      +------+--+  Exit    |
                | Running +----------+
                +---------+
 */
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

impl TaskControlBlock {
    // Constructs TaskControlBlock for the app, including:
    // - Initializes memory set(mapping and page table) in the user space.
    // - Allocates specific memory as kernel stack for this app in the kernel
    // space.
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        debug!("Initializing task control block for app: {}", app_id);
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_ctx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_ADDR).into())
            .unwrap();
        let task_status = TaskStatus::Ready;
        let (kernel_stack_bottom, kernel_stack_top) =
            kernel_stack_position(app_id);
        debug!(
            "kernel bottom {:#x} and top {:#x}",
            kernel_stack_bottom, kernel_stack_top
        );
        KERNEL_SPACE.exclusive_access().push(
            VirtPageNumRange::new_from_va(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
            ),
            Mapping::new_framed(),
            MapPermission::R | MapPermission::W,
            None,
        );
        let task_control_block = Self {
            ctx: TaskContext::goto_trap_return(kernel_stack_top),
            status: task_status,
            memory_set,
            trap_ctx_ppn,
            base_size: user_sp,
        };
        debug!("trap_return {:?}", task_control_block.ctx);
        let trap_ctx = task_control_block.get_trap_ctx();
        *trap_ctx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        debug!("Initialization of app {} done", app_id);
        task_control_block
    }
    fn get_trap_ctx(&self) -> &'static mut TrapContext {
        self.trap_ctx_ppn.get_mut()
    }
}
