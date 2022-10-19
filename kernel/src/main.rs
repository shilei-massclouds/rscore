/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

#![no_std]
#![no_main]
#![feature(naked_functions, asm_sym, asm_const)]
#![feature(default_alloc_error_handler)]

mod lang;
mod vm;
mod config_generated;

#[macro_use]
mod align;

#[path = "arch/riscv64/mod.rs"]
mod arch;

use core::sync::atomic::AtomicI32;

static HART_LOTTERY: AtomicI32 = AtomicI32::new(0);

static BOOT_HARTID: usize = 0;
static DTB_PA: usize = 0;
