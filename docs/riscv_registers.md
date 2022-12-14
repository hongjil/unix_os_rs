# RISC-V registers cheatsheet
[RISC-V Assembly Programmer's Manual](https://github.com/riscv-non-isa/riscv-asm-manual/blob/master/riscv-asm.md)
## General-purpose registers 

reg    | name  | saver  | description
-------|-------|--------|------------
x0     | zero  |        | hardwired zero
x1     | ra    | caller | return address
x2     | sp    | callee | stack pointer
x3     | gp    |        | global pointer
x4     | tp    |        | thread pointer
x5-7   | t0-2  | caller | temporary registers
x8     | s0/fp | callee | saved register / frame pointer
x9     | s1    | callee | saved register
x10-11 | a0-1  | caller | function arguments / return values
x12-17 | a2-7  | caller | function arguments
x18-27 | s2-11 | callee | saved registers
x28-31 | t3-6  | caller | temporary registers
pc     |       |        | program counter

## Supervisor Control/status register
[Reference](https://riscv.org/wp-content/uploads/2017/05/riscv-privileged-v1.10.pdf)

### Controlled by hardware
These CSRs are expected to be adapted automatically when a trap happened.
- `sstatus`: Keeps track of the processor’s current **operating states**. Among the bits:
    - `SPP` bit indicates the privilege level at which a hart[^1] was executing before entering supervisor mode. When a trap is taken, SPP is set to 0 if the trap originated from user mode, or 1 otherwise.
    - `SIE` bit indicates whether interrupts are enabled or disabled in supervisor mode. When it's clear, interrupts are not taken while in supervisor mode. When the hart is running in user-mode, the value on `SIE` bit is ignored, and supervisor interrupts are always enabled. The supervisor can disable or enable individual interrupt source using `sie` register.
    - `PSIE` bit whether supervisor interrupts were enabled prior to trapping into supervisor mode. When a trap happens, then `SIE` -> `SPIE` and `0` -> `SIE`. When `SRET` executed, restored back the two bits. 
      > The nested interrupt/trap hence is disabled by since we set `0` to `SIE` when trapping to the supervisor mode.

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
- `satp`: Supervisor Address Translation and Protection; Consists of 3 parts:
    - `MODE`: the translation scheme
    - `PPN`:  The physical page number of the root page table.
    - `ASID`: *TBD*

[^1]: hart means "hardware thread"