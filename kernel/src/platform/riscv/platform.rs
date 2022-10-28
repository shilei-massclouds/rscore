/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::slice;

use crate::{BootContext, dprint, CRITICAL, INFO};
use crate::errors::ErrNO;
use crate::vm::bootreserve::boot_reserve_init;

/* all of the configured memory arenas */
pub const NUM_ARENAS: usize = 16;

#[derive(Copy, Clone)]
pub struct ArenaInfo {
    name: [char; 16],

    flags: usize,

    base: usize,
    size: usize,
}

pub fn platform_early_init(ctx: &mut BootContext) -> Result<(), ErrNO> {
    /* initialize the boot memory reservation system */
    boot_reserve_init(ctx)?;

    /* discover memory ranges */
    parse_dtb(ctx);

    /* find memory ranges to use if one is found. */
    /*
    for (size_t i = 0; i < arena_count; i++) {
        if (!have_limit || status != ZX_OK) {
            pmm_add_arena(&mem_arena[i]);
        }
    }
    */

    Ok(())
}

pub fn parse_dtb(ctx: &mut BootContext) -> Result<(), ErrNO> {
    /* Early scan of device tree from init memory */
    let dtb_va = ctx.pa_to_va(ctx.dtb_pa);
    dprint!(CRITICAL, "HartID {:x} DTB 0x{:x} -> 0x{:x}\n",
            ctx.hartid, ctx.dtb_pa, dtb_va);

    let magic_ptr = dtb_va as *const u32;
    unsafe {
        let a: u32 = *magic_ptr;
        dprint!(INFO, "dtb magic 0x{:x}\n", a);
    }

    dprint!(CRITICAL, "No DTB passed to the kernel\n");
    Err(ErrNO::BadDTB)
}
