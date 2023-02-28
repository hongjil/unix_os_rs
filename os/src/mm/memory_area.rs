use super::{
    address::*,
    frame_allocator::{frame_alloc, FrameTracker},
    page_table::{PTEFlags, PageTable},
};
use crate::error::Result;
use crate::utils::StepByOne;
use crate::{
    config::{
        MEMORY_END, PAGE_SIZE, TRAMPOLINE_ADDR, TRAP_CONTEXT_ADDR,
        USER_STACK_SIZE,
    },
    error::KernelError,
};
use alloc::collections::{BTreeMap, LinkedList};
use core::arch::asm;
use core::cmp::max;
use riscv::register::satp;

// The MemoryArea includes the information about a consecutive memory segment
// given the context(page table).
#[derive(Debug)]
pub struct MemoryArea {
    vpn_range: VirtPageNumRange,
    // The mapping schema for this map area.
    mapping: Mapping,
    map_perm: MapPermission,
}

#[derive(Debug)]
pub enum Mapping {
    // If the mapping schema is identical, VPN==PPN.
    Identical,
    // If the mapping schema is Framed, allocating a frame as PPN for it.
    Framed(BTreeMap<VirtPageNum, FrameTracker>),
}

bitflags! {
    pub struct MapPermission: u8 {
        // The bit index matches what we have in PageTableEntry.
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

impl MemoryArea {
    pub fn new(
        vpn_range: VirtPageNumRange,
        mapping: Mapping,
        map_perm: MapPermission,
    ) -> Self {
        Self {
            vpn_range,
            mapping,
            map_perm,
        }
    }
    pub fn map(&mut self, vpn: VirtPageNum) -> PhysPageNum {
        self.mapping.map(vpn)
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        self.mapping.unmap(vpn)
    }
    pub fn get_vpn_range(&self) -> VirtPageNumRange {
        self.vpn_range
    }
    pub fn get_map_perm(&self) -> MapPermission {
        self.map_perm
    }
}
impl Mapping {
    pub fn new_framed() -> Self {
        Mapping::Framed(BTreeMap::new())
    }
}
