mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use crate::sync::UPSafeCell;
pub use address::*;
use alloc::sync::Arc;
use lazy_static::*;
pub use memory_set::{MapPermission, Mapping, MemorySet};
pub use page_table::{translated_byte_buffer, translated_mut_byte_buffer};

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe { UPSafeCell::new(MemorySet::new_kernel()) });
}

pub fn init() {
    heap_allocator::init_heap();
    // The frame allocator depends on the heap allocator.
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
    #[cfg(debug_assertions)]
    memory_set::remap_kernel_test();
}
