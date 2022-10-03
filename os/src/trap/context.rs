use riscv::register::sstatus::{self, Sstatus, SPP};

// A context tracks the registers for trapping. The layout of this struct
// matches how it stores registers into stack specified in ./trap.S;
// Check docs/riscv_registers.md for details.
#[repr(C)]
pub struct TrapContext {
    // General purpose registers:
    // - x[2] is the user's stack pointer.
    pub x: [usize; 32],
    // Supervisor CSRs:
    // The supervisor status: either Supervisor or User.
    pub sstatus: Sstatus,
    // The address where the trap happens.
    pub sepc: usize,
}

impl TrapContext {
    // Sets x[2] which is stack pointer of userspace.
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    // Initializes the trap context before application running.
    // We assume there is a trap right before applications start to run.
    pub fn app_init_context(entry: usize, sp: usize) -> TrapContext {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut context = TrapContext {
            x: [0; 32],
            sstatus: sstatus,
            sepc: entry,
        };
        context.set_sp(sp);
        context
    }
}
