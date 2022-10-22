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
mod timer;
mod trap;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    if cfg!(debug_assertions) {
        println!("[kernel] Debugging enabled");
    } else {
        println!("[kernel] Debugging disabled");
    }
    println!("[kernel] Hello, world!");
    trap::init();
    loader::load_apps();
    println!("[kernel] setting up timer interrupt");
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    println!("[kernel] Start running first task");
    task::run_first_task();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
