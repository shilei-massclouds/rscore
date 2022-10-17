/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::arch::asm;
use super::csr::*;

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
        options(noreturn)
    )
}
