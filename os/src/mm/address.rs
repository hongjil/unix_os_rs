// This file contains the basic structs in address accessing and translation.
use super::page_table::*;
use crate::config::*;
use crate::utils::*;
use core::fmt::{self, Debug, Formatter};
use core::ops;

const PA_WIDTH_SV39: usize = 56;
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VA_LEVEL_SV39: usize = 3;
const VA_BITS_PER_LEVEL_SV39: usize = 9;
const VPN_WIDTH_SV39: usize = VA_LEVEL_SV39 * VA_BITS_PER_LEVEL_SV39;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

pub type VirtPageNumRange = SimpleRange<VirtPageNum>;

// Debugging
impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}
impl Debug for VirtPageNumRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "VPN Range:({:#x}, {:#x})",
            self.get_start().0,
            self.get_end().0
        ))
    }
}

impl VirtPageNumRange {
    pub fn new_from_va(start_va: VirtAddr, end_va: VirtAddr) -> Self {
        VirtPageNumRange::new(start_va.floor(), end_va.ceil())
    }
}

// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
// T -> usize: T.0
// usize -> T: usize.into()
impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1usize << VPN_WIDTH_SV39) - 1))
    }
}
impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1usize << VA_WIDTH_SV39) - 1))
    }
}
impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        Self(v & ((1usize << PA_WIDTH_SV39) - 1))
    }
}
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1usize << PPN_WIDTH_SV39) - 1))
    }
}
impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}
impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        v.0
    }
}
impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}

// Translation between page number and address.
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        Self(v.0 >> PAGE_SIZE_BITS)
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        Self(v.0 >> PAGE_SIZE_BITS)
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

// Customized implementation for each struct.
impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        (self.0 / PAGE_SIZE).into()
    }
    pub fn ceil(&self) -> PhysPageNum {
        ((self.0 + PAGE_SIZE - 1) / PAGE_SIZE).into()
    }
}
impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        (self.0 >> PAGE_SIZE_BITS).into()
    }
    pub fn ceil(&self) -> VirtPageNum {
        ((self.0 + (1usize << PAGE_SIZE_BITS) - 1) >> PAGE_SIZE_BITS).into()
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}
impl PhysPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            core::slice::from_raw_parts_mut(
                pa.0 as *mut PageTableEntry,
                PT_SIZE,
            )
        }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = self.clone().into();
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, 1usize << 12)
        }
    }
    // FIXME: Adapt the interface to be safer.
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = self.clone().into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }
}
impl VirtPageNum {
    pub fn indexes(&self) -> [usize; VA_LEVEL_SV39] {
        let mut vpn = self.0;
        let mut idx = [0usize; VA_LEVEL_SV39];
        for i in 0..VA_LEVEL_SV39 {
            idx[i] = vpn & ((1 << VA_BITS_PER_LEVEL_SV39) - 1);
            vpn >>= VA_BITS_PER_LEVEL_SV39;
        }
        idx
    }
}
impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 = self.0 + 1;
    }
}
impl ops::Add<usize> for PhysPageNum {
    type Output = PhysPageNum;
    fn add(self, _rhs: usize) -> PhysPageNum {
        let num: usize = self.into();
        (num + _rhs).into()
    }
}
