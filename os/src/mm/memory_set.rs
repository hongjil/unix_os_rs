// The memory set is a set of memory areas, sharing the same page table.
// It's an abstraction of context and address space(i.e. kernel or user apps).

use super::{
    address::*,
    frame_allocator::{frame_alloc, FrameTracker},
    page_table::{PTEFlags, PageTable},
};
use crate::config::{
    MEMORY_END, PAGE_SIZE, TRAMPOLINE_ADDR, TRAP_CONTEXT_ADDR, USER_STACK_SIZE,
};
use crate::sync::UPSafeCell;
use crate::utils::StepByOne;
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::arch::asm;
use core::cmp::max;
use riscv::register::satp;

pub struct MemorySet {
    page_table: Arc<UPSafeCell<PageTable>>,
    areas: Vec<MapArea>,
}
// The MapArea includes the information about a consecutive memory segment
// given the context(page table).
pub struct MapArea {
    vpn_range: VirtPageNumRange,
    // The mapping schema for this map area.
    mapping: Mapping,
    map_perm: MapPermission,
    page_table: Arc<UPSafeCell<PageTable>>,
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

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

impl Mapping {
    pub fn new_framed() -> Self {
        Mapping::Framed(BTreeMap::new())
    }
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            page_table: Arc::new(unsafe { UPSafeCell::new(PageTable::new()) }),
            areas: Vec::new(),
        }
    }
    pub fn push(
        &mut self,
        vpn_range: VirtPageNumRange,
        mapping: Mapping,
        map_perm: MapPermission,
        data: Option<&[u8]>,
    ) {
        let pt = self.page_table.clone();
        let new_area = MapArea::new(pt, vpn_range, mapping, map_perm, data);
        for area in &self.areas {
            if new_area.vpn_range.is_overlapped(area.vpn_range) {
                println!(
                    "The new area {:?} conflicts with the existing one {:?}",
                    new_area.vpn_range, area.vpn_range
                );
            }
        }
        self.areas.push(new_area);
    }

    pub fn activate(&self) {
        let satp = self.page_table.exclusive_access().token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        self.page_table
            .exclusive_access()
            .translate(vpn)
            .map(|e| e.ppn())
    }

    pub fn token(&self) -> usize {
        self.page_table.exclusive_access().token()
    }

    // Trampoline is where context switch happens; The page table is changed
    // hence we need to make sure every context(either user or kernel) should
    // have the same mapping otherwise the $pc might access invalid memory
    // address.
    fn map_trampoline(&self) {
        self.page_table.exclusive_access().map(
            VirtAddr::from(TRAMPOLINE_ADDR).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    // Identical mapping is required for "smooth transition" when the kernel
    // page table is activated.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new();
        memory_set.map_trampoline();
        println!("[kernel] Initializing kernel space");
        println!(
            "[kernel] Mapping .text section [{:#x}, {:#x})",
            stext as usize, etext as usize
        );
        memory_set.push(
            VirtPageNumRange::new_from_va(
                (stext as usize).into(),
                (etext as usize).into(),
            ),
            Mapping::Identical,
            MapPermission::R | MapPermission::X,
            None,
        );
        println!(
            "[kernel] Mapping .rodata section [{:#x}, {:#x})",
            srodata as usize, erodata as usize
        );
        memory_set.push(
            VirtPageNumRange::new_from_va(
                (srodata as usize).into(),
                (erodata as usize).into(),
            ),
            Mapping::Identical,
            MapPermission::R,
            None,
        );
        println!(
            "[kernel] Mapping .data section [{:#x}, {:#x})",
            sdata as usize, edata as usize
        );
        memory_set.push(
            VirtPageNumRange::new_from_va(
                (sdata as usize).into(),
                (edata as usize).into(),
            ),
            Mapping::Identical,
            MapPermission::R | MapPermission::W,
            None,
        );
        println!(
            "[kernel] Mapping .bss section [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        memory_set.push(
            VirtPageNumRange::new_from_va(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
            ),
            Mapping::Identical,
            MapPermission::R | MapPermission::W,
            None,
        );
        println!(
            "[kernel] Mapping physical memory [{:#x}, {:#x})",
            ekernel as usize, MEMORY_END as usize
        );
        memory_set.push(
            VirtPageNumRange::new_from_va(
                (ekernel as usize).into(),
                (MEMORY_END as usize).into(),
            ),
            Mapping::Identical,
            MapPermission::R | MapPermission::W,
            None,
        );
        memory_set
    }

    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new();
        memory_set.map_trampoline();
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        assert_eq!(
            elf_header.pt1.magic,
            [0x7f, 0x45, 0x4c, 0x46],
            "invalid elf!"
        );
        let mut max_end_vpn = VirtPageNum(0);
        for ph in elf.program_iter() {
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr =
                    ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let vpn_range = VirtPageNumRange::new_from_va(start_va, end_va);
                max_end_vpn = max(max_end_vpn, vpn_range.get_end());
                memory_set.push(
                    vpn_range,
                    Mapping::new_framed(),
                    map_perm,
                    Some(
                        &elf.input[ph.offset() as usize
                            ..(ph.offset() + ph.file_size()) as usize],
                    ),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            VirtPageNumRange::new_from_va(
                user_stack_bottom.into(),
                user_stack_top.into(),
            ),
            Mapping::new_framed(),
            MapPermission::R | MapPermission::W | MapPermission::U,
            None,
        );
        // map TrapContext
        memory_set.push(
            VirtPageNumRange::new_from_va(
                TRAP_CONTEXT_ADDR.into(),
                TRAMPOLINE_ADDR.into(),
            ),
            Mapping::new_framed(),
            MapPermission::R | MapPermission::W,
            None,
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }
}

impl Mapping {
    pub fn map(&mut self, vpn: VirtPageNum) -> PhysPageNum {
        match self {
            Mapping::Identical => PhysPageNum::from(vpn.0),
            Mapping::Framed(ref mut frames) => match frames.get(&vpn) {
                Some(frame) => frame.0,
                None => {
                    let frame: FrameTracker = frame_alloc().unwrap();
                    let ret = frame.0;
                    frames.insert(vpn, frame);
                    ret
                }
            },
        }
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        match self {
            Mapping::Identical => {}
            Mapping::Framed(ref mut frames) => {
                frames.remove(&vpn);
            }
        }
    }
}

impl MapArea {
    pub fn new(
        pt: Arc<UPSafeCell<PageTable>>,
        vpn_range: VirtPageNumRange,
        mapping: Mapping,
        map_perm: MapPermission,
        data: Option<&[u8]>,
    ) -> Self {
        // 0. Construct a MapArea instance.
        let mut ret = Self {
            vpn_range,
            mapping,
            map_perm,
            page_table: pt.clone(),
        };
        // 1. Populate itself to the page table.
        let pte_flags = PTEFlags::from_bits(map_perm.bits).unwrap();
        let mut page_table = pt.exclusive_access();
        for vpn in ret.vpn_range {
            let ppn = ret.mapping.map(vpn);
            page_table.map(vpn, ppn, pte_flags);
        }
        // 2. Optionally, if there is data, copy the the data into the MapArea.
        if let Some(data) = data {
            let mut start: usize = 0;
            let mut current_vpn = ret.vpn_range.get_start();
            let len = data.len();
            loop {
                let src = &data[start..len.min(start + PAGE_SIZE)];
                let dst = &mut page_table
                    .translate(current_vpn)
                    .unwrap()
                    .ppn()
                    .get_bytes_array()[..src.len()];
                dst.copy_from_slice(src);
                start += PAGE_SIZE;
                if start >= len {
                    break;
                }
                current_vpn.step();
                if current_vpn >= ret.vpn_range.get_end() {
                    panic!(
                        "[kernel] Insufficient memory for data with size: \
                        {:?} vs {:?}",
                        ret.vpn_range.into_iter().count() * PAGE_SIZE,
                        data.len()
                    );
                }
            }
        }
        ret
    }
}

impl Drop for MapArea {
    fn drop(&mut self) {
        let mut page_table = self.page_table.exclusive_access();
        for vpn in self.vpn_range {
            self.mapping.unmap(vpn);
            page_table.unmap(vpn);
        }
    }
}

#[allow(unused)]
pub fn remap_kernel_test() {
    let mut kernel_space = crate::mm::KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr =
        ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space
            .page_table
            .exclusive_access()
            .translate(mid_text.floor())
            .unwrap()
            .writable(),
        false
    );
    assert_eq!(
        kernel_space
            .page_table
            .exclusive_access()
            .translate(mid_rodata.floor())
            .unwrap()
            .writable(),
        false,
    );
    assert_eq!(
        kernel_space
            .page_table
            .exclusive_access()
            .translate(mid_data.floor())
            .unwrap()
            .executable(),
        false,
    );
    println!("remap_test passed!");
}
