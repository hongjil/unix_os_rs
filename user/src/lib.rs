#![feature(linkage)]
#![feature(panic_info_message)]
#![no_std]
#[macro_use]

pub mod console;
mod lang_items;
mod syscall;

// The "real" entry point for each user binary.
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    // bss clearing should be done by OS :(
    clear_bss();

    exit(main());
    panic!("unreachable after sys_exit!");
}

// This is a trick to jump to the real "main"s in the bin folder.
// Since the linkage is "weak", it will be overwritten by binary's one.
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
