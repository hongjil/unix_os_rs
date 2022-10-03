# RISC-V registers cheatsheet

## General-purpose registers 
[Calling convention](https://en.wikichip.org/wiki/risc-v/registers)

## Supervisor Control/status register
[Reference](https://riscv.org/wp-content/uploads/2017/05/riscv-privileged-v1.10.pdf)

### Controlled by hardware
These CSRs are expected to be adapted automatically when a trap happened.
- `sstatus`: Keeps track of the processor’s current **operating states**. 
    - Among the bits, the `SPP` bit indicates the privilege level at which a hart[^1] was executing before entering supervisor
mode. When a trap is taken, SPP is set to 0 if the trap originated from user mode, or 1 otherwise.
- `sepc`: When a trap is taken into S-mode, sepc is written with the **virtual address** of the instruction that encountered the exception.
- `scause`/`stval`: 
  - When a trap is taken into S-mode, `scause` is written with a **code** indicating the event that caused the trap.
  - When a trap is taken into S-mode, `stval` is written with **exception-specific information** to assist software in handling the trap.
    - It seems like providing auxiliary information and it depends on case by case; sometimes an address or some bits.
### Controlled by software
These CSRs are expected to be initialized or adapted by OS.
- `stvec`: Holds trap vector **configuration**, consisting of a vector base address `BASE` and a vector mode `MODE`.
  - When `MODE`=`Direct` => All exceptions set pc to `BASE`.
  - When `MODE`=`Vectored` => Asynchronous interrupts set pc to `BASE+4×cause`
- `sscratch`: Hold a pointer to the hart-local **supervisor context** while the hart is executing user code. At the beginning of a trap handler, `sscratch` is swapped with a user register to provide an initial working register.  
    - In this project, `sscratch` points to the kernel stack while `sp` points to the user stack and they will swap during trap.

[^1]: hart means "hardware thread"