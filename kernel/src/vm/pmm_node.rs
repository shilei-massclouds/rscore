/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use alloc::vec::Vec;
use super::pmm_arena::PmmArena;
use crate::MAX_ARENAS;
use crate::{
    ArenaInfo, dprint, INFO, CRITICAL, BootReserveRange,
    PAGE_SIZE, IS_ALIGNED, IS_PAGE_ALIGNED, ErrNO,
};
use crate::lib::list::List;
use crate::vm::page::vm_page_t;
use core::sync::atomic::{AtomicU64, Ordering};

/* per numa node collection of pmm arenas and worker threads */
pub struct PmmNode {
    arenas: Vec<PmmArena>,

    arena_cumulative_size: usize,

    /* Free pages where !loaned. */
    free_count  : AtomicU64,
    free_list   : List<vm_page_t>,
}

impl PmmNode {
    pub fn new() -> PmmNode {
        PmmNode {
            arenas: Vec::<PmmArena>::with_capacity(MAX_ARENAS),

            arena_cumulative_size: 0,

            free_count  : AtomicU64::new(0),
            free_list   : List::new(),
        }
    }

    /* during early boot before threading exists. */
    pub fn add_arena(&mut self, info: ArenaInfo,
                     reserve_ranges: &Vec<BootReserveRange>)
        -> Result<(), ErrNO> {
        dprint!(INFO, "PMM: adding arena '{}' base {:x} size {:x}\n",
                info.name, info.base, info.size);

        if !IS_PAGE_ALIGNED!(info.base) ||
           !IS_PAGE_ALIGNED!(info.size) ||
           (info.size == 0) {
            return Err(ErrNO::BadAlign);
        }

        let mut arena = PmmArena::new(info);
        if let Err(e) = arena.init(self, reserve_ranges) {
            dprint!(CRITICAL, "PMM: pmm_add_arena failed {:?}\n", e);
            /* but ignore this failure */
            return Ok(());
        }

        dprint!(INFO, "Adding arena '{}' ...\n", arena.name());

        self.arena_cumulative_size += arena.size();

        /* insert arena in ascending order of its base address */
        let mut pos = 0;
        for a in &(self.arenas) {
            if arena.base() < a.base() {
                return Ok(self.arenas.insert(pos, arena));
            }
            pos += 1;
        }

        Ok(self.arenas.push(arena))
    }

    pub fn add_free_pages(&mut self, list: &mut List<vm_page_t>) {
        self.free_count.fetch_add(list.len() as u64, Ordering::Relaxed);
        self.free_list.append(list);
        // free_pages_evt_.Signal();

        dprint!(INFO, "free count now {}\n",
                self.free_count.load(Ordering::Relaxed));
    }
}
