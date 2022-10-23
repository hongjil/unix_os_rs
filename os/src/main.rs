#![no_main]
#![no_std]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
mod config;
#[macro_use]
mod console;
mod lang_items;
mod loader;
mod mm;
mod sbi;
mod stack_trace;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

extern crate alloc;

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
    println!("[kernel] Initializing trap handling");
    trap::init();
    println!("[kernel] Initializing heap allocator");
    mm::init_heap();
    loader::load_apps();
    println!("[kernel] Setting up timer interrupt");
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    println!("[kernel] Start running tasks");
    task::run_first_task();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}
