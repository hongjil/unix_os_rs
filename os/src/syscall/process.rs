use crate::config::PAGE_SIZE;
use crate::mm::*;
use crate::task::*;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// Creates a mapping area in the current user context.
/// Args:
///     - usize: the start of virtual address
///     - len: the size of map area
///     - prot: The first three bit is valid only, corresponding to RWX perm.
/// Return 0 if success and -1 if fail.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    if (prot & !0x7) > 0 || (prot & 0x7) == 0 || start % PAGE_SIZE != 0 {
        return -1;
    }
    let result = current_user_memory_set().exclusive_access().push_area(
        MemoryArea::new(
            VirtPageNumRange::new_from_va(start.into(), (start + len).into()),
            Mapping::new_framed(),
            MapPermission::U
                | MapPermission::from_bits_truncate((prot << 1) as u8),
        ),
        true,
        None,
    );
    match result {
        Ok(_) => 0,
        Err(err) => {
            println!(
                "[kernel] sys_mmap({}, {}, {:#b}) error: {}",
                start, len, prot, err
            );
            -1
        }
    }
}

/// Creates and inserts a mapping area in the current user context.
/// Args:
///     - usize: the start of virtual address
///     - len: the size of map area
///     - prot: The first three bit is valid only, corresponding to RWX perm.
/// Return 0 if success and -1 if fail.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let result = current_user_memory_set().exclusive_access().drop_area(
        VirtPageNumRange::new_from_va(start.into(), (start + len).into()),
    );
    match result {
        Ok(_) => 0,
        Err(err) => {
            println!("[kernel] sys_munmap({}, {}) error: {}", start, len, err);
            -1
        }
    }
}
