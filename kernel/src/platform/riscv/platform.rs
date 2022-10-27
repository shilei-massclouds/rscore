/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::{BootContext, dprint, CRITICAL};
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
    dprint!(CRITICAL, "Hart-ID {} DTB pa 0x{:x}\n",
            ctx.hartid, ctx.dtb_pa);
    //early_init_dt_scan(DTB_PA)?;
    /*
    if (early_init_dt_scan(dtb_early_va)) {
        const char *name = of_flat_dt_get_machine_name();

        if (name) {
            pr_info("Machine model: %s\n", name);
            dump_stack_set_arch_desc("%s (DT)", name);
        }
        return;
    }
    */

    dprint!(CRITICAL, "No DTB passed to the kernel\n");
    Err(ErrNO::BadDTB)
}
