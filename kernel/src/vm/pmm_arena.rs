/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::mem;
use crate::{
    ArenaInfo, PmmNode, ErrNO, dprint, CRITICAL, INFO,
    PAGE_SIZE, ROUNDUP_PAGE_SIZE, ROUNDUP,
};
use crate::vm::page::vm_page_t;

pub struct PmmArena<'a> {
    info: &'a ArenaInfo<'a>,
}

impl PmmArena<'_> {
    pub fn new<'a>(info: &'a ArenaInfo<'a>) -> PmmArena<'a> {
        PmmArena {
            info,
        }
    }

    pub fn init(&self, pmm_node: &PmmNode) -> Result<(), ErrNO> {
        /* allocate an array of pages to back this one */
        let page_count = self.info.size / PAGE_SIZE;
        let vm_page_sz = mem::size_of::<vm_page_t>();
        let page_array_size = ROUNDUP_PAGE_SIZE!(page_count*vm_page_sz);

        /* if the arena is too small to be useful, bail */
        if page_array_size >= self.info.size {
            dprint!(CRITICAL,
                    "PMM: arena too small to hold page array ({:x})\n",
                    self.info.size);
            return Err(ErrNO::LackBuf);
        }

        dprint!(INFO, "page array size {:x}\n", page_array_size);

        Ok(())
    }
}
