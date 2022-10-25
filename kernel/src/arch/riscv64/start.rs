/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::arch::asm;
use super::csr::*;
use super::defines::*;
use super::mmu::{
    riscv64_boot_map, riscv64_setup_trampoline, SWAPPER_PG_DIR,
    TRAMPOLINE_SATP, SWAPPER_SATP,
    PAGE_KERNEL, PAGE_KERNEL_EXEC
};
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
fn _start() -> ! {
    asm!(
        /* Make sure hartid < CONFIG_NR_CPUS */
        "li t0, {NR_CPUS}
         blt a0, t0, 1f
        0:
         wfi
         j 0b
        1:",

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

        /* Pick one hart to run the main boot sequence.
         * Since early OpenSBI(version < v0.7) has no HSM extension,
         * a lottery system is required: secondary harts spinwait
         * for the unique winner to finish most jobs. */
        "la a3, {HART_LOTTERY}
         li a2, 1
         amoadd.w a3, a2, (a3)
         bnez a3, 2f",

        /* Save hart ID and DTB physical address */
        "la a2, {BOOT_HARTID}
         sd a0, (a2)
         la a2, {DTB_PA}
         sd a1, (a2)",

        /* Clear BSS for flat non-ELF images */
        "la a3, __bss_start
         la a4, _end
         ble a4, a3, 4f
        3:
         sd zero, (a3)
         add a3, a3, 8
         blt a3, a4, 3b
        4:",

        /* Setup stack for boot hart */
        "li t0, {stack_size}
         la sp, {boot_stack}
         add sp, sp, t0",

        "j {start_kernel}",

        /* For secondary start */
        "2:
         wfi
         j 2b",

        NR_CPUS = const NR_CPUS,
        SR_FS = const SR_FS,
        HART_LOTTERY = sym crate::HART_LOTTERY,
        BOOT_HARTID = sym crate::BOOT_HARTID,
        DTB_PA = sym crate::DTB_PA,
        stack_size = const _CONFIG_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        start_kernel = sym start_kernel,
        options(noreturn)
    );
}

#[naked]
#[repr(align(4))]
#[link_section = ".head.text"]
unsafe extern "C"
fn relocate_enable_mmu() {
    asm!(
        /* Calculate offset between va and pa for kernel */
        "li a1, {kernel_base}
         la a2, __code_start
         sub a1, a1, a2",

        /* Relocate return address */
        "add ra, ra, a1",

        /* Point stvec to virtual address of intruction
         * after satp write */
        "la a2, 1f
         add a2, a2, a1
         csrw stvec, a2",

        /* Compute satp for swapper page tables,
         * but don't load it yet */
        "la a2, {swapper_satp}
         ld a2, (a2)",

        /*
         * Load trampoline page directory, which will cause us to trap
         * to stvec if VA != PA, or simply fall through if VA == PA.
         * We need a full fence here because boot_map()
         * just wrote these PTEs and we need to ensure
         * the new translations are in use.
         */
        "la a0, {trampoline_satp}
         ld a0, (a0)
         sfence.vma
         csrw satp, a0",    /* Turn on MMU based on trampoline pg dir */

        /* Switch point from pa to va. */
        ".align 2
        1:
         la a0, 2f
         csrw stvec, a0",   /* Set trap vector to spin forever temporarily. */

        /* Reload the global pointer */
        ".option push
         .option norelax
         la gp, __global_pointer$
         .option pop",

        /* Reset stack pointer */
        "li t0, {stack_size}
         la sp, {boot_stack}
         add sp, sp, t0",

        /* Setup thread pointer */
        /* Todo: la tp, init_task */

        /*
         * Switch to swapper page tables.
         * A full fence is necessary in order to avoid using
         * the trampoline translations, which are only correct for
         * the first superpage.
         * Fetching the fence is guaranteed to work
         * because that first superpage is translated the same way.
         */
        "csrw satp, a2
         sfence.vma
         ret",

        /* Loop forever for debug. */
        ".align 2
        2:
         wfi
         j 2b",

        kernel_base = const KERNEL_BASE,
        trampoline_satp = sym TRAMPOLINE_SATP,
        swapper_satp = sym SWAPPER_SATP,
        stack_size = const _CONFIG_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        options(noreturn)
    );
}

unsafe extern "C"
fn start_kernel() {
    /* map a large run of physical memory
     * at the base of the kernel's address space */
    let ret = riscv64_boot_map(&mut SWAPPER_PG_DIR,
                               KERNEL_ASPACE_BASE, 0, ARCH_PHYSMAP_SIZE,
                               PAGE_KERNEL);
    if let Err(_) = ret {
        return;
    }

    /* Symbol __code_start and _end comes from kernel.ld */
    let kernel_base_phys = __code_start as usize;
    let kernel_size = (_end as usize) - kernel_base_phys +
        BOOT_HEAP_SIZE;

    /* map the kernel to a fixed address */
    /* map the boot heap that just follows kernel address */
    let ret = riscv64_boot_map(&mut SWAPPER_PG_DIR,
                               KERNEL_BASE,
                               kernel_base_phys, kernel_size,
                               PAGE_KERNEL_EXEC);
    if let Err(_) = ret {
        return;
    }

    /* Setup trampoline: mapping at phys -> phys */
    riscv64_setup_trampoline(kernel_base_phys);

    /* Enable MMU */
    relocate_enable_mmu();

    /* Set the per cpu pointer for cpu 0 */

    /* Enter main */
    crate::lk_main();
}

//unsafe extern "C"
//fn secondary_park() {
//    /* Lack SMP support or have too many harts, so part this hart! */
//    loop {
//        asm!("wfi")
//    }
//}
