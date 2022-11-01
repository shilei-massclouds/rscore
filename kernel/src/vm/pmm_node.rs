/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use alloc::vec::Vec;
use super::pmm_arena::PmmArena;
use crate::MAX_ARENAS;
use crate::{
    ArenaInfo, dprint, INFO, CRITICAL,
    PAGE_SIZE, IS_ALIGNED, IS_PAGE_ALIGNED, ErrNO,
};

/* per numa node collection of pmm arenas and worker threads */
pub struct PmmNode<'a> {
    arenas: Vec<PmmArena<'a>>,
}

impl PmmNode<'_> {
    pub fn new() -> PmmNode<'static> {
        PmmNode {
            arenas: Vec::<PmmArena>::with_capacity(MAX_ARENAS),
        }
    }

    /* during early boot before threading exists. */
    pub fn add_arena(&self, info: &ArenaInfo) -> Result<(), ErrNO> {
        dprint!(INFO, "PMM: adding arena '{}' base {:x} size {:x}\n",
                info.name, info.base, info.size);

        if !IS_PAGE_ALIGNED!(info.base) ||
           !IS_PAGE_ALIGNED!(info.size) ||
           (info.size == 0) {
            return Err(ErrNO::BadAlign);
        }

        let arena = PmmArena::new(info);
        if let Err(e) = arena.init(self) {
            dprint!(CRITICAL, "PMM: pmm_add_arena failed {:?}\n", e);
            /* but ignore this failure */
            return Ok(());
        }

        dprint!(INFO, "Adding arena {} ...\n", info.name);

        /*
  if (status != ZX_OK) {
    // leaks boot allocator memory
    arena->~PmmArena();
  }
  */

        Ok(())
    }
}
