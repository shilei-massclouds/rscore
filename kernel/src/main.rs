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
mod arch;
mod config_generated;

use core::sync::atomic::AtomicI32;

static HART_LOTTERY: AtomicI32 = AtomicI32::new(0);
