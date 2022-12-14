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
mod types;
mod lib;
mod boot;
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

use types::*;
use core::sync::atomic::AtomicI32;
use crate::arch::sbi::*;
use crate::arch::defines::*;
use crate::platform::platform_early_init;
//use crate::kernel::thread::thread_init_early;
use crate::lib::debuglog::debuglog::*;
use alloc::vec::Vec;
use crate::vm::bootreserve::{MAX_RESERVES, BootReserveRange};
use crate::vm::pmm::{MAX_ARENAS, ArenaInfo};
use crate::vm::pmm_node::PmmNode;
use crate::errors::ErrNO;
use crate::arch::periphmap::{PeriphRange, MAX_PERIPH_RANGES};
use crate::vm::vm::vm_init_preheap;
use crate::vm::vm_aspace::VmAspace;
use crate::lib::list::List;

pub struct BootContext {
    hartid: usize,
    dtb_pa: paddr_t,
    reserve_ranges: Vec<BootReserveRange>,
    /* peripheral ranges are allocated below the kernel image. */
    periph_ranges: Vec<PeriphRange>,
    periph_base_virt: vaddr_t,
    /* The (currently) one and only pmm node */
    pmm_node: PmmNode,
    aspaces: List<VmAspace>,
}

impl BootContext {
    pub fn new(hartid: usize, dtb_pa: paddr_t) -> BootContext {
        BootContext {
            hartid,
            dtb_pa,
            reserve_ranges:
                Vec::<BootReserveRange>::with_capacity(MAX_RESERVES),
            periph_ranges:
                Vec::<PeriphRange>::with_capacity(MAX_PERIPH_RANGES),
            periph_base_virt: 0,
            pmm_node: PmmNode::new(),
            aspaces: List::<VmAspace>::new(),
        }
    }

    /*
    fn _kernel_pa_to_va(&self, pa: usize) -> usize {
        pa + (KERNEL_BASE - KERNEL_BASE_PHYS)
    }
    */
}

static HART_LOTTERY: AtomicI32 = AtomicI32::new(0);

fn kernel_base_phys() -> usize {
    unsafe { __kernel_base_phys }
}

fn kernel_size() -> usize {
    (_end as usize) - (__code_start as usize)
}

/* called from arch code */
fn lk_main(hartid: usize, dtb_pa: paddr_t) -> Result<(), ErrNO> {
    let mut ctx = BootContext::new(hartid, dtb_pa);

    /* get us into some sort of thread context so Thread::Current works. */
    //thread_init_early();

    /* bring the debuglog up early so we can safely printf */
    dlog_init_early();

    /* we can safely printf now since we have the debuglog,
     * the current thread set which holds (a per-line buffer),
     * and global ctors finished (some of the printf machinery
     * depends on ctors right now). */
    dprint!(ALWAYS, "printing enabled\n");

    dprint!(ALWAYS, "params [{:x}, {:x}] dtb_phys: {:x} ... \n",
            kernel_base_phys(), kernel_size(), ctx.dtb_pa);
    platform_early_init(&mut ctx)?;

    // DriverHandoffEarly(*gPhysHandoff);
    // lk_primary_cpu_init_level(LK_INIT_LEVEL_PLATFORM_EARLY,
    //                           LK_INIT_LEVEL_ARCH_PREVM - 1);

    /* At this point, the kernel command line and serial are set up. */

    dprint!(INFO, "\nwelcome to rscon\n\n");
    dprint!(SPEW, "KASLR: .text section at 0x{:x}\n", kernel_base_phys());

    /* Perform any additional arch and platform-specific set up
     * that needs to be done before virtual memory or the heap are set up. */
    dprint!(SPEW, "initializing arch pre-vm\n");
    // arch_prevm_init();
    // lk_primary_cpu_init_level(LK_INIT_LEVEL_ARCH_PREVM,
    //                           LK_INIT_LEVEL_PLATFORM_PREVM - 1);
    dprint!(SPEW, "initializing platform pre-vm\n");
    // platform_prevm_init();
    // lk_primary_cpu_init_level(LK_INIT_LEVEL_PLATFORM_PREVM,
    //                           LK_INIT_LEVEL_VM_PREHEAP - 1);

    /* perform basic virtual memory setup */
    dprint!(SPEW, "initializing vm pre-heap\n");
    vm_init_preheap();
    // lk_primary_cpu_init_level(LK_INIT_LEVEL_VM_PREHEAP,
    //                           LK_INIT_LEVEL_HEAP - 1);

    Ok(())
}
