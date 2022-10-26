/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::arch::asm;
use super::csr::*;
use super::defines::*;
use super::mmu::{
    riscv64_boot_map, riscv64_setup_mmu_mode, SWAPPER_PG_DIR,
    SWAPPER_SATP,
    PAGE_KERNEL, PAGE_KERNEL_EXEC
};
use crate::config_generated::*;

#[link_section = ".bss..page_aligned"]
static mut BOOT_STACK: [u8; _CONFIG_STACK_SIZE] =
    [0u8; _CONFIG_STACK_SIZE];

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

        /* Make sure hartid < CONFIG_NR_CPUS */
        "li t0, {NR_CPUS}
         bge a0, t0, 1f",

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

        /* Loop forever due to bad hartid. */
        "1:
         wfi
         j 1b",

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
         * Switch to swapper page tables.
         * Load swapper page directory, which will cause us to trap to
         * stvec if VA != PA, or simply fall through if VA == PA.
         * We need a full fence here because riscv64_boot_map() just
         * wrote these PTEs and we need to ensure the new translations
         * are in use.
         */
        "sfence.vma
         csrw satp, a2",

        /* Switch point from pa to va. */
        ".align 2
        1:",

        /* Set trap vector to spin forever for debug. */
        "la a0, 2f
         csrw stvec, a0",

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

        "ret",

        /* Loop forever for debug. */
        ".align 2
        2:
         wfi
         j 2b",

        kernel_base = const KERNEL_BASE,
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

    /* Setup value for register satp */
    riscv64_setup_mmu_mode();

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
