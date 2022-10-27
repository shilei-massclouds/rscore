/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::BootContext;
use crate::errors::ErrNO;
use crate::vm::bootreserve::boot_reserve_init;

pub fn platform_early_init(ctx: &mut BootContext) -> Result<(), ErrNO> {
  /* initialize the boot memory reservation system */
  boot_reserve_init(ctx)
}
