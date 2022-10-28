/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

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
    parse_dtb(ctx)?;

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

fn fdt_get_u32(dtb_va: usize, offset: usize) -> u32 {
    let ptr = (dtb_va + offset) as *const u32;
    unsafe {
        u32::from_be(*ptr)
    }
}

const FDT_MAGIC: u32 = 0xd00dfeed;

const FDT_MAGIC_OFFSET: usize = 0;
const FDT_TOTALSIZE_OFFSET: usize = 4;

fn early_init_dt_verify(dtb_va: usize) -> Result<(), ErrNO> {
    if dtb_va == 0 {
        return Err(ErrNO::NullDTB);
    }

    /* check device tree validity */
    if fdt_get_u32(dtb_va, FDT_MAGIC_OFFSET) != FDT_MAGIC {
        return Err(ErrNO::BadDTB);
    }

    Ok(())
}

fn early_init_dt_scan(dtb_va: usize) -> Result<(), ErrNO> {
    early_init_dt_verify(dtb_va)?;

    let totalsize = fdt_get_u32(dtb_va, FDT_TOTALSIZE_OFFSET);
    dprint!(INFO, "dtb totalsize 0x{:x}\n", totalsize);
    /*
    early_init_dt_scan_nodes();
    return true;
    */
    Ok(())
}

pub fn parse_dtb(ctx: &mut BootContext) -> Result<(), ErrNO> {
    /* Early scan of device tree from init memory */
    let dtb_va = ctx.pa_to_va(ctx.dtb_pa);
    dprint!(CRITICAL, "HartID {:x} DTB 0x{:x} -> 0x{:x}\n",
            ctx.hartid, ctx.dtb_pa, dtb_va);

    early_init_dt_scan(dtb_va)?;

    dprint!(CRITICAL, "No DTB passed to the kernel\n");
    Err(ErrNO::BadDTB)
}
