/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::{
    BootContext, dprint, INFO,
    PAGE_SIZE, IS_PAGE_ALIGNED,
};
use crate::errors::ErrNO;
use crate::arch::mmu::{PAGE_IOREMAP, riscv64_boot_map_v};

pub const MAX_PERIPH_RANGES : usize = 4;

pub struct PeriphRange {
    pub base_phys:  usize,
    pub base_virt:  usize,
    pub length:     usize,
}

pub fn add_periph_range(ctx: &mut BootContext,
                        base_phys: usize, length: usize)
    -> Result<(), ErrNO> {

    if ctx.periph_ranges.len() >= MAX_PERIPH_RANGES {
        return Err(ErrNO::OutOfRange);
    }

    if !IS_PAGE_ALIGNED!(base_phys) || !IS_PAGE_ALIGNED!(length) {
        return Err(ErrNO::BadAlign);
    }

    dprint!(INFO, "periphmap: {:x}\n", ctx.periph_base_virt);

    riscv64_boot_map_v(ctx.periph_base_virt, base_phys, length,
                       PAGE_IOREMAP)?;

    ctx.periph_ranges.push(
        PeriphRange {
            base_phys,
            base_virt: ctx.periph_base_virt,
            length
        }
    );

    ctx.periph_base_virt += length;

    Ok(())
}
