pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

// Execute following commands to get the CLOCK_FREQ for QEMU:
//   qemu-system-riscv64 -machine virt,dumpdtb=dump.dtb
//   dtc -o dump.dts dump.dtb
// Then check `timebase-frequency` in cpus.
pub const CLOCK_FREQ: usize = 10_000_000;
pub const MICRO_PER_SEC: usize = 1_000_000;
