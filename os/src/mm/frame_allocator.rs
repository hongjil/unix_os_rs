// The frame allocator manages the allocation and deallocation of physical page.
use super::address::*;
use crate::config::MEMORY_END;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::*;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}
#[derive(Debug)]
pub struct FrameTracker(pub PhysPageNum);
pub struct StackFrameAllocator {
    // Maintains a range of PhysPages which is not allocated at all.
    current: PhysPageNum, // The start index of available PhysPages;
    end: PhysPageNum,     // The end index of available PhysPages;
    // A list contains recycled PhysPages.
    recycled: Vec<PhysPageNum>,
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

// We only expose alloc method and we depend on RAII scheme to dealloc.
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self(ppn)
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.0);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, current: PhysPageNum, end: PhysPageNum) {
        debug!("Frame allocator current: {:?} end:{:?}", current, end);
        self.current = current;
        self.end = end;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        StackFrameAllocator {
            current: PhysPageNum::from(0),
            end: PhysPageNum::from(0),
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(page) = self.recycled.pop() {
            Some(page)
        } else {
            if self.current == self.end {
                None
            } else {
                let result = self.current;
                self.current = self.current + 1;
                Some(result)
            }
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        if ppn >= self.current
            || self.recycled.iter().find(|&v| *v == ppn).is_some()
        {
            panic!("Frame ppn {:?} has not been allocated!", ppn);
        }
        self.recycled.push(ppn);
    }
}
