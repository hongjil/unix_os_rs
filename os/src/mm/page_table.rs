// This file includes PageTable and PageTableEntry. Here we use SV39 scheme
// which includes 12 bits of page size and 3-level page table, where the page
// directory in each level takes 9 bits.
//
// Check docs/pics/page_table.svg for details.
use super::address::*;
use super::frame_allocator::*;
use crate::utils::StepByOne;
use alloc::vec::*;
use bitflags::*;
use core::fmt::{self, Debug, Formatter};

use super::frame_allocator::FrameTracker;

const SATP_MODE_SV39: usize = 8usize << 60;
const PTE_WIDTH_SV39: usize = 54;
const PTE_PAGE_NUM_START_BIT: usize = 10;
// The number of PTE in page table each level.
pub const PT_SIZE: usize = 1 << 9;

// Each page table entry(PTE) is a usize; where the first low 8 bit are the
// flags to indicate its state.
bitflags! {
    pub struct PTEFlags: u8 {
        // If the PTE is valid
        const V = 1 << 0;
        // Permission to Read/write/execute.
        // When all 3 are clear and the page is valid, then this
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        // If this page allow to access in User mode.
        const U = 1 << 4;
        // Ignore this for now.
        const G = 1 << 5;
        // If this page is adapted after the last time clear.
        const A = 1 << 6;
        // If this page is dirty after the last time clear.
        const D = 1 << 7;
    }
}
// Each page table entry consist of two parts:
// - [0, 7] PTE flags.
// - [8, 53] The physical page num given with vpn_idx in this page directory.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry(usize);
// The page table has 3-level page directory and each page directory has 512
// (2^9) page table entries.
pub struct PageTable {
    // The physical page num of the root level page directory.
    root_ppn: PhysPageNum,
    // Keeps track of each frames allocated from FrameAllocator.
    frames: Vec<FrameTracker>,
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PTE:{:?}|{:?}", self.ppn(), self.flags()))
    }
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self((ppn.0 << PTE_PAGE_NUM_START_BIT) | flags.bits as usize)
    }
    pub fn empty() -> Self {
        Self(0)
    }
    pub fn ppn(&self) -> PhysPageNum {
        ((self.0 & ((1usize << PTE_WIDTH_SV39) - 1)) >> PTE_PAGE_NUM_START_BIT)
            .into()
    }
    pub fn flags(&self) -> PTEFlags {
        let pte = self.0;
        let flag_value = pte as u8;
        PTEFlags::from_bits(flag_value).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

impl PageTable {
    pub fn new() -> Self {
        let root_page_directory = frame_alloc().unwrap();
        PageTable {
            root_ppn: root_page_directory.0,
            frames: vec![root_page_directory],
        }
    }
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp),
            frames: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        self.root_ppn.0 | SATP_MODE_SV39
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| pte.clone())
    }

    // Populates PTEs in this page table given the mapping
    // intention(VPN, PPN & flags).
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_mut_pte(vpn);
        assert!(!pte.is_valid(), "vpn {:#x} is mapped before mapping", vpn.0);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    // Removes PTEs in this page table given the VPN.
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_mut_pte(vpn);
        assert!(
            pte.is_valid(),
            "vpn {:#x} is invalid before unmapping",
            vpn.0
        );
        *pte = PageTableEntry::empty();
    }

    // Returns the mutable PTE given a VPN.
    // When there is a missing of page directory, automatically allocates one
    // physical page from frame allocator.
    fn find_mut_pte(&mut self, vpn: VirtPageNum) -> &mut PageTableEntry {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, vpn_id) in idxs.iter().enumerate().rev() {
            let pte = &mut ppn.get_pte_array()[*vpn_id];
            if i == 0 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let page_directory = frame_alloc().unwrap();
                *pte = PageTableEntry::new(page_directory.0, PTEFlags::V);
                assert!(pte.is_valid());
                self.frames.push(page_directory);
            }
            ppn = pte.ppn();
        }
        result.unwrap()
    }

    // Returns the PTE given a VPN.
    // When there is a missing of page directory, return none.
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&PageTableEntry> = None;
        for (i, vpn_id) in idxs.iter().enumerate().rev() {
            let pte = &mut ppn.get_pte_array()[*vpn_id];
            if i == 0 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
}

// Returns a list of memory slice in physical address given the root address
// of page table.
pub fn translated_byte_buffer(
    token: usize,
    ptr: *const u8,
    len: usize,
) -> Vec<&'static [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start_va = VirtAddr::from(ptr as usize);
    let end_va = VirtAddr::from(ptr as usize + len);
    let mut v = Vec::new();
    while start_va < end_va {
        let mut start_vpn = start_va.floor();
        let ppn = page_table.translate(start_vpn).unwrap().ppn();
        start_vpn.step();
        let cur_end_va: VirtAddr = end_va.min(start_vpn.into());
        v.push(
            &ppn.get_bytes_array()
                [start_va.page_offset()..cur_end_va.page_offset()],
        );
        start_va = cur_end_va;
    }
    v
}

// Returns a list of mutable  memory slice in physical address given the root
// address of page table.
pub fn translated_mut_byte_buffer(
    token: usize,
    ptr: *const u8,
    len: usize,
) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start_va = VirtAddr::from(ptr as usize);
    let end_va = VirtAddr::from(ptr as usize + len);
    let mut v = Vec::new();
    while start_va < end_va {
        let mut start_vpn = start_va.floor();
        let ppn = page_table.translate(start_vpn).unwrap().ppn();
        start_vpn.step();
        let cur_end_va: VirtAddr = end_va.min(start_vpn.into());
        v.push(
            &mut ppn.get_bytes_array()
                [start_va.page_offset()..cur_end_va.page_offset()],
        );
        start_va = cur_end_va;
    }
    v
}
