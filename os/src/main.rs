#![no_main]
#![no_std]
#![feature(panic_info_message)]
#[macro_use]
mod console;
mod config;
mod lang_items;
mod loader;
mod sbi;
mod stack_trace;
mod sync;
mod syscall;
mod task;
mod trap;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");
    trap::init();
    loader::load_apps();
    task::run_first_task();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
