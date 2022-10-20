/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::arch::asm;
use core::sync::atomic::Ordering;
use super::csr::*;
use super::defines::*;
use super::mmu;
use crate::vm::bootalloc::*;
use crate::config_generated::*;

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
    extern "C" {
        static mut __bss_start: u64;
        static mut _end: u64;
    }
    r0::zero_bss(&mut __bss_start, &mut _end);

    /* Setup stack for boot hart */
    setup_boot_stack();

    /* The boot allocator only works in the early stage */
    let mut bootalloc = BootAlloc::new();

    /* Map a large run of physical memory
     * at the base of the kernel's address space */
    let ret = mmu::riscv64_boot_map(&mut bootalloc, &mut (mmu::SWAPPER_PG_DIR),
                                    KERNEL_ASPACE_BASE, 0, ARCH_PHYSMAP_SIZE,
                                    0);
    if let Err(_) = ret {
        return;
    }
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
    #[link_section = ".bss..page_aligned"]
    static mut BOOT_STACK: [u8; _CONFIG_STACK_SIZE]
        = [0u8; _CONFIG_STACK_SIZE];

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
