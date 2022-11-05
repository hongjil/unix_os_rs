.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
.section .text.trampoline
.globl __alltraps
.globl __restore
.align 2

# The sscratch stores the address of TrapContext in user space when entering
# trap handling
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext, sscratch->user stack
    # save general-purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it into TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp; Note that the the kernel stack address is based on
    # kernel space satp.
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0
    # refresh TLB
    sfence.vma
    # jump to trap_handler
    jr t1

# When the __restore is expected to be called:
# - case1: start running the first task by __restore
# - case2: back to U after handling trap
#
# Arguements:
# - a0: *TrapContext in user space
# - a1: user space satp
__restore:
    # swtich to user space
    csrw satp, a1
    # refresh TLB
    sfence.vma
    # Load the address of TrapContext to sscratch for the next trap handling
    csrw sscratch, a0
    # Load trap context to sp
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general-purpuse registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret