/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::arch::asm;
use core::sync::atomic::Ordering;
use super::csr::*;
use super::defines::*;
use super::mmu::{
    riscv64_boot_map, riscv64_setup_trampoline, SWAPPER_PG_DIR,
    TRAMPOLINE_SATP, SWAPPER_SATP,
    PAGE_KERNEL, PAGE_KERNEL_EXEC
};
use crate::vm::bootalloc::*;
use crate::config_generated::*;

#[link_section = ".bss..page_aligned"]
static mut BOOT_STACK: [u8; _CONFIG_STACK_SIZE] = [0u8; _CONFIG_STACK_SIZE];

/*
 * Entry
 * Refer to '_start' in kernel for riscv.
 */
#[naked]
#[no_mangle]
#[link_section = ".head.text"]
unsafe extern "C"
fn _start(hartid: usize, dtb_pa: usize) -> ! {
    asm!(
        "j {start_kernel}",
        start_kernel = sym start_kernel,
        options(noreturn)
    )
}

#[link_section = ".head.text"]
unsafe extern "C"
fn relocate_enable_mmu() {
    asm!(
        /* Calculate diffence between va and pa for kernel */
        ".align 2
         li t2, {kernel_base}
         la t3, __code_start
         sub t2, t2, t3",

        /* Relocate return address */
        "add ra, ra, t2",

        /*
         * Set satp for swapper page directory,
         * but don't use it now!
         */
        "la t1, {swapper_satp}
         ld t1, (t1)",

        /*
         * Set satp for trampoline page directory and turn on MMU.
         * We need a full fence here because boot_map() just wrote these PTEs and
         * we need to ensure the new translations are in use.
         */
        "la t0, {trampoline_satp}
         ld t0, (t0)
         sfence.vma
         csrw satp, t0",    /* Turn on MMU based on trampoline pg dir */

        /* PC = next_PC + diffence */
        "la t0, 1f
         add t0, t0, t2
         jr t0
        .align 2
        1:
         sfence.vma
         csrw satp, t1",    /* Turn on MMU based on swapper pg dir */

        /* Reload the global pointer */
        ".option push
         .option norelax
         la gp, __global_pointer$
         .option pop",

        kernel_base = const KERNEL_BASE,
        trampoline_satp = sym TRAMPOLINE_SATP,
        swapper_satp = sym SWAPPER_SATP,
    );

    /* Reset stack pointer */
    setup_boot_stack();
}

unsafe extern "C"
fn start_kernel(hartid: usize) {
    prepare(hartid);

    /*
     * Since early OpenSBI(version < v0.7) has no HSM extension,
     * a lottery system is required: secondary harts spinwait for
     * the unique winner to finish most jobs.
     */
    if crate::HART_LOTTERY.fetch_add(1, Ordering::Relaxed) != 0 {
        return secondary_park();
    }

    /* Clear BSS */
    r0::zero_bss(&mut (__bss_start as u64), &mut (_end as u64));

    /* Setup stack for boot hart */
    setup_boot_stack();

    /* The boot allocator only works in the early stage */
    let mut bootalloc = BootAlloc::new();

    /* map a large run of physical memory
     * at the base of the kernel's address space */
    let ret = riscv64_boot_map(&mut bootalloc, &mut SWAPPER_PG_DIR,
                               KERNEL_ASPACE_BASE, 0, ARCH_PHYSMAP_SIZE,
                               PAGE_KERNEL);
    if let Err(_) = ret {
        return;
    }

    /* Symbol __code_start and _end comes from kernel.ld */
    let kernel_base_phys = __code_start as usize;
    let kernel_size = (_end as usize) - kernel_base_phys;

    /* map the kernel to a fixed address */
    let ret = riscv64_boot_map(&mut bootalloc, &mut SWAPPER_PG_DIR,
                               KERNEL_BASE, kernel_base_phys, kernel_size,
                               PAGE_KERNEL_EXEC);
    if let Err(_) = ret {
        return;
    }

    /* Setup trampoline: mapping at phys -> phys */
    riscv64_setup_trampoline(kernel_base_phys);

    /* Enable MMU */
    relocate_enable_mmu();
}

unsafe extern "C"
fn prepare(hartid: usize) {
    asm!(
        /* Mask all interrupts */
        "csrw sie, zero
         csrw sip, zero",

        /* Load the global pointer */
        ".option push
         .option norelax
         la gp, __global_pointer$
         .option pop",

        /*
         * Disable FPU to detect illegal usage of
         * floating point in kernel space
         */
        "li t0, {SR_FS}
         csrc sstatus, t0",

        /* Save hart ID and DTB physical address */
        "la a2, {BOOT_HARTID}
         sd a0, (a2)
         la a2, {DTB_PA}
         sd a1, (a2)",

        SR_FS = const SR_FS,
        BOOT_HARTID = sym crate::BOOT_HARTID,
        DTB_PA = sym crate::DTB_PA,
    );

    if hartid >= _CONFIG_NR_CPUS {
        return secondary_park();
    }

    /* From now on, don't use hartid(a0) and dtb_pa(a1) again!
     * Some crates may clobber a0 or a1 unexpectedly */
}

unsafe extern "C"
fn setup_boot_stack() {
    asm! (
        "li t0, {stack_size}
         la sp, {boot_stack}
         add sp, sp, t0",
        stack_size = const _CONFIG_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
    )
}

unsafe extern "C"
fn secondary_park() {
    /* Lack SMP support or have too many harts, so part this hart! */
    loop {
        asm!("wfi")
    }
}
