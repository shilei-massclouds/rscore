/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::{
    dprint, INFO, BootContext,
};
use crate::errors::ErrNO;

pub const NUM_RESERVES: usize = 64;

#[derive(Copy, Clone)]
pub struct BootReserveRange {
    pa: usize,
    len: usize,
}

pub fn boot_reserve_init(ctx: &mut BootContext) -> Result<(), ErrNO> {
    /* add the kernel to the boot reserve list */
    boot_reserve_add_range(ctx.kernel_base_phys, ctx.kernel_size, ctx)
}

/* given two offset/length pairs, determine if they overlap at all */
#[inline]
fn intersects(offset1: usize, len1: usize,
              offset2: usize, len2: usize) -> bool {
    /* Can't overlap a zero-length region. */
    if len1 == 0 || len2 == 0 {
        return false;
    }

    if offset1 <= offset2 {
        /* doesn't intersect, 1 is completely below 2 */
        if offset1 + len1 <= offset2 {
            return false;
        }
    } else if offset1 >= offset2 + len2 {
        /* 1 is completely above 2 */
        return false;
    }

    true
}

fn boot_reserve_add_range(pa: usize, len: usize, ctx: &mut BootContext)
    -> Result<(), ErrNO> {

    dprint!(INFO, "PMM: boot reserve add [0x{:x}, 0x{:x}]\n",
            pa, pa + len - 1);

    let end = pa + len - 1;

    /* insert into the list, sorted */
    let mut i = 0;
    while i < ctx.reserve_ranges.len() {
        if intersects(ctx.reserve_ranges[i].pa,
                      ctx.reserve_ranges[i].len,
                      pa, len) {
            return Err(ErrNO::BadRange);
        }

        if ctx.reserve_ranges[i].pa > end {
            break;
        }

        i += 1;
    }

    let range = BootReserveRange{pa: pa, len: len};
    ctx.reserve_ranges.insert(i, range);

    dprint!(INFO, "Boot reserve #range {}\n", ctx.reserve_ranges.len());
    Ok(())
}
