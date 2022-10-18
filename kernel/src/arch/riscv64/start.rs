/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::arch::asm;
use core::sync::atomic::Ordering;
use super::csr::*;
use crate::config_generated::*;

/*
 * Entry
 * Refer to '_start' in kernel for riscv.
 */
#[naked]
#[no_mangle]
#[link_section = ".head.text"]
unsafe extern "C"
fn _start(hartid: usize, device_tree_paddr: usize) -> ! {
    asm!(
        "j {start_kernel}",
        start_kernel = sym start_kernel,
        options(noreturn)
    )
}

unsafe extern "C"
fn start_kernel(hartid: usize) {
    prepare();

    /*
     * Since early OpenSBI(version < v0.7) has no HSM extension,
     * a lottery system is required: secondary harts spinwait for
     * the unique winner to finish most jobs.
     */
    if hartid >= _CONFIG_NR_CPUS {
        return secondary_park();
    }

    if crate::HART_LOTTERY.fetch_add(1, Ordering::Relaxed) != 0 {
        return secondary_park();
    }
}

unsafe extern "C"
fn prepare() {
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

        SR_FS = const SR_FS,
    )
}

unsafe extern "C"
fn secondary_park() {
    /* Lack SMP support or have too many harts, so part this hart! */
    loop {
        asm!("wfi")
    }
}
