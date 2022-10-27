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
#![feature(repr_simd)]
#![feature(allow_internal_unstable)]
#![feature(fmt_internals)]

mod lang;
mod errors;
mod lib;
//mod kernel;
mod vm;
mod config_generated;

#[macro_use]
mod align;

#[path = "arch/riscv64/mod.rs"]
mod arch;

#[path = "platform/riscv/platform.rs"]
mod platform;

extern crate alloc;

use core::sync::atomic::AtomicI32;
use crate::arch::sbi::*;
use crate::platform::platform_early_init;
//use crate::kernel::thread::thread_init_early;
use crate::lib::debuglog::debuglog::*;
use alloc::vec::Vec;
use crate::vm::bootreserve::{NUM_RESERVES, BootReserveRange};
use crate::errors::ErrNO;

pub struct BootContext {
    kernel_base_phys: usize,
    kernel_size: usize,
    reserve_ranges: Vec<BootReserveRange>,
}

impl BootContext {
    pub fn new(kernel_base_phys: usize,
               kernel_size: usize) -> BootContext {

        BootContext {
            kernel_base_phys,
            kernel_size,
            reserve_ranges:
                Vec::<BootReserveRange>::with_capacity(NUM_RESERVES),
        }
    }
}

static HART_LOTTERY: AtomicI32 = AtomicI32::new(0);

static BOOT_HARTID: usize = 0;
static DTB_PA: usize = 0;

/* called from arch code */
fn lk_main(kernel_base_phys: usize, kernel_size: usize)
    -> Result<(), ErrNO> {
    let mut ctx = BootContext::new(kernel_base_phys, kernel_size);

    /* get us into some sort of thread context so Thread::Current works. */
    //thread_init_early();

    /* bring the debuglog up early so we can safely printf */
    dlog_init_early();

    /* we can safely printf now since we have the debuglog,
     * the current thread set which holds (a per-line buffer),
     * and global ctors finished (some of the printf machinery
     * depends on ctors right now). */
    dprint!(ALWAYS, "printing enabled\n");

    platform_early_init(&mut ctx)?;
    dprint!(ALWAYS, "prepare memory[{:x}, {:x}] ... \n",
            kernel_base_phys, kernel_size);

    Ok(())
}
