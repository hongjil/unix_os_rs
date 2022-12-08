pub mod context;

use crate::config::{TRAMPOLINE_ADDR, TRAP_CONTEXT_ADDR};
use crate::sbi::shutdown;
use crate::{syscall::syscall, task::*, timer::set_next_trigger};
use core::arch::{asm, global_asm};
use riscv::register::sie;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    stval, stvec,
};

global_asm!(include_str!("trap.S"));

// Initialize the stvec register so that it knows where to jump
// when a trap happens.
pub fn init() {
    set_kernel_trap_entry();
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE_ADDR as usize, TrapMode::Direct);
    }
}

pub fn set_shutdown_trap_entry() {
    unsafe {
        stvec::write(shutdown as usize, TrapMode::Direct);
    }
}

pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_ctx_ptr = TRAP_CONTEXT_ADDR;
    let user_satp = current_user_memory_set().exclusive_access().token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE_ADDR;
    unsafe {
        // https://doc.rust-lang.org/reference/inline-assembly.html
        asm!(
            // Clear i-cache
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_ctx_ptr,
            in("a1") user_satp,
            options(noreturn)
        )
    }
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    let scause = scause::read();
    let stval = stval::read();
    panic!("A trap from kernel: {:?} with {:?}", scause.cause(), stval);
}

// Handles an interrupt, exception or system call.
// Jumps from the __alltrap and return to __restore in trap.S
#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let ctx = current_trap_ctx();
    let scause = scause::read();
    let stval = stval::read();
    debug!("A trap from user: {:?} with {:?}", scause.cause(), stval);
    match scause.cause() {
        // Triggered from user space, executing system call.
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4;
            ctx.x[10] =
                syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] Store pageFault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.", stval, ctx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!(
                "[kernel] IllegalInstruction in application, core dumped."
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] Load pageFault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.", stval, ctx.sepc);
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // Note that we should not have nested interrupt by default; As a
            // result, the interrupts will "stacked" instead. i.e. handling
            // traps one-by-one.
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

pub use context::TrapContext;
