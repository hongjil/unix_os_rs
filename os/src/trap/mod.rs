pub mod context;

use crate::{syscall::syscall, task::run_next_app};
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap},
    stval, stvec,
};

global_asm!(include_str!("trap.S"));

// Initialize the stvec register so that it knows where to jump
// when a trap happens.
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

// Handles an interrupt, exception or system call.
// Jumps from the __alltrap and return to __restore in trap.S
#[no_mangle]
fn trap_handler(ctx: &mut context::TrapContext) -> &mut context::TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        // Triggered from user space, executing system call.
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4;
            ctx.x[10] = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, core dumped.");
            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            run_next_app();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    ctx
}

pub use context::TrapContext;
