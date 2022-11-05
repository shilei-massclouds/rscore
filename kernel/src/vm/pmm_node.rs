/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use super::pmm_arena::PmmArena;
use crate::MAX_ARENAS;
use crate::{
    ArenaInfo, dprint, INFO, CRITICAL, BootReserveRange, paddr_t,
    PAGE_SIZE, IS_ALIGNED, IS_PAGE_ALIGNED, ErrNO, ROUNDDOWN,
};
use crate::lib::list::{List, Linked};
use crate::vm::page::vm_page_t;

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

    pub fn alloc_range(&self, paddr: paddr_t, count: usize,
                       list: &mut List<vm_page_t>)
        -> Result<(), ErrNO> {
        dprint!(INFO, "address {:x}, count {}\n", paddr, count);

        if count == 0 {
            return Ok(());
        }

        let mut paddr = ROUNDDOWN!(paddr, PAGE_SIZE);

        let mut allocated = 0;

        //AutoPreemptDisabler preempt_disable;
        //Guard<Mutex> guard{&lock_};

        /* walk through the arenas,
         * looking to see if the physical page belongs to it */
        'next: for a in &(self.arenas) {
            while allocated < count {
                if !a.address_in_arena(paddr) {
                    continue 'next;
                }

                if let Some(mut page) = a.find_specific(paddr) {
                    /* And we never allocate loaned pages
                     * for caller of AllocRange() */
                    unsafe {
                        if page.as_ref().is_free() &&
                           !page.as_ref().is_loaned() {
                            page.as_mut().delete_from_list();
                            //AllocPageHelperLocked(page);
                            list.add_tail(page);
                            allocated += 1;
                            //DecrementFreeCountLocked(1);
                        }
                    }
                }

                paddr += PAGE_SIZE;
            }

            if allocated >= count {
                break;
            }
        }

        if allocated != count {
            /* we were not able to allocate the entire run,
             * free these pages */
            //FreeListLocked(list);
            return Err(ErrNO::NotFound);
        }

        dprint!(INFO, "########## alloc range ok!\n");
        Ok(())
    }
}
