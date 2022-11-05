pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

// Execute following commands to get the CLOCK_FREQ for QEMU:
//   qemu-system-riscv64 -machine virt,dumpdtb=dump.dtb
//   dtc -o dump.dts dump.dtb
// Then check `timebase-frequency` in cpus.
pub const CLOCK_FREQ: usize = 10_000_000;
pub const MICRO_PER_SEC: usize = 1_000_000;

pub const MEMORY_END: usize = 0x80800000;

pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1usize << PAGE_SIZE_BITS;

// The trampoline is placed in the last page.
pub const TRAMPOLINE_ADDR: usize = usize::MAX - PAGE_SIZE + 1;
// The trap context is placed in the second last page.
pub const TRAP_CONTEXT_ADDR: usize = TRAMPOLINE_ADDR - PAGE_SIZE;

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAP_CONTEXT_ADDR - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
