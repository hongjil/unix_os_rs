#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn zero_init() -> Self {
        TaskContext {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    pub fn init(kernel_sp: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        TaskContext {
            ra: __restore as usize,
            sp: kernel_sp,
            s: [0; 12],
        }
    }
}
