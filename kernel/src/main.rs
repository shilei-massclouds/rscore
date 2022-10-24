/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

#![no_std]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![feature(default_alloc_error_handler)]
#![feature(fn_align)]

mod lang;
mod errors;
//mod lib;
//mod kernel;
mod vm;
mod config_generated;

#[macro_use]
mod align;

#[path = "arch/riscv64/mod.rs"]
mod arch;

use core::sync::atomic::AtomicI32;
use crate::arch::sbi::*;
//use crate::kernel::thread::thread_init_early;
//use crate::lib::debuglog::debuglog::*;

static HART_LOTTERY: AtomicI32 = AtomicI32::new(0);

static BOOT_HARTID: usize = 0;
static DTB_PA: usize = 0;

/* called from arch code */
fn lk_main() {
    /* get us into some sort of thread context so Thread::Current works. */
    //thread_init_early();

    /* bring the debuglog up early so we can safely printf */
    //dlog_init_early();

    /* we can safely printf now since we have the debuglog,
     * the current thread set which holds (a per-line buffer),
     * and global ctors finished (some of the printf machinery
     * depends on ctors right now). */
    //dprint!(ALWAYS, "printing enabled\n");
    console_putchar('A');
}
