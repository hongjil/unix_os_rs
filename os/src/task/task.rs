use super::TaskContext;

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub ctx: TaskContext,
    pub status: TaskStatus,
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
