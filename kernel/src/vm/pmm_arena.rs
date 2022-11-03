/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::mem;
use alloc::vec::Vec;
use crate::{
    ArenaInfo, PmmNode, ErrNO, dprint, CRITICAL, INFO,
    PAGE_SIZE, ROUNDUP_PAGE_SIZE, ROUNDUP, PAGE_ALIGN, ALIGN,
    BootReserveRange, paddr_t,
};
use crate::vm::page::{vm_page_t, vm_page};
use crate::vm::vm_page_state;
use crate::vm::vm_page_state::vm_page_state_t;
use crate::vm::physmap::paddr_to_physmap;
use crate::vm::bootreserve::boot_reserve_range_search;
use crate::lib::list::List;
use core::ptr::NonNull;

struct PageArray {
    start:      paddr_t,
    len:        usize,
    obj_size:   usize,
}

impl PageArray {
    fn new() -> Self {
        Self {
            start:  0,
            len:    0,
            obj_size: mem::size_of::<vm_page_t>(),
        }
    }

    fn init(&mut self, start: paddr_t, len: usize) {
        self.start = start;
        self.len = len;
    }

    fn get_page(&self, index: usize) -> Option<NonNull<vm_page_t>> {
        let ptr = index * self.obj_size + self.start;
        if ptr >= (self.start + self.len) {
            return None;
        }

        NonNull::<vm_page_t>::new(ptr as *mut vm_page_t)
    }

    fn init_page(&self, index: usize, paddr: paddr_t)
        -> Result<(), ErrNO> {

        let mut page = self.get_page(index)
            .ok_or_else(|| ErrNO::NoMem)?;

        unsafe {
            page.as_mut().init(paddr);
        }
        Ok(())
    }

    fn set_page_state(&self, index: usize, state: vm_page_state_t)
        -> Result<(), ErrNO> {
        let mut page = self.get_page(index)
            .ok_or_else(|| ErrNO::NoMem)?;

        unsafe { page.as_mut().set_state(state); }
        Ok(())
    }
}

pub struct PmmArena {
    info: ArenaInfo,
    page_array_: PageArray,
}

impl PmmArena {
    pub fn new(info: ArenaInfo) -> PmmArena {
        PmmArena {
            info,
            page_array_: PageArray::new(),
        }
    }

    /* accessors */
    fn _info(&self) -> &ArenaInfo {
        &(self.info)
    }

    pub fn name(&self) -> &str {
        self.info.name.as_str()
    }

    pub fn base(&self) -> paddr_t {
        self.info.base
    }

    pub fn size(&self) -> usize {
        self.info.size
    }

    fn _flags(&self) -> u32 {
        self.info.flags
    }

    pub fn init(&mut self, pmm_node: &mut PmmNode,
                reserve_ranges: &Vec<BootReserveRange>)
        -> Result<(), ErrNO> {

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

        /* allocate a chunk to back the page array out of
         * the arena itself, near the top of memory */
        let mut range = BootReserveRange::default();
        boot_reserve_range_search(self.info.base, self.info.size,
                                  page_array_size,
                                  reserve_ranges,
                                  &mut range)?;

        if range.pa < self.info.base || range.len > page_array_size {
            return Err(ErrNO::OutOfRange);
        }

        dprint!(INFO, "arena for base {:x} size {:x}\n",
                range.pa, range.len);

        let page_array_va = paddr_to_physmap(range.pa);
        self.page_array_.init(page_array_va, page_array_size);

        /* |page_count| pages in the state FREE */
        vm_page::add_to_initial_count(vm_page_state::FREE, page_count);

        /* compute the range of the array that backs the array itself */
        let array_start_index =
            (PAGE_ALIGN!(range.pa) - self.info.base) / PAGE_SIZE;
        let array_end_index =
            array_start_index + page_array_size / PAGE_SIZE;

        dprint!(INFO, "array_start_index {}, array_end_index {}\n",
                array_start_index, array_end_index);

        if array_start_index >= page_count ||
           array_end_index > page_count {
            return Err(ErrNO::BadRange);
        }

        dprint!(INFO, "init page_array ...\n");

        /* add all pages that aren't part of the page array
         * to the free list pages */
        let mut list = List::new();

        let mut i = 0;
        while i < page_count {
            let paddr = self.info.base + i * PAGE_SIZE;
            self.page_array_.init_page(i, paddr)?;

            if i >= array_start_index && i < array_end_index {
                self.page_array_
                    .set_page_state(i, vm_page_state::WIRED)?;
            } else {
                let page = self.page_array_.get_page(i)
                    .ok_or_else(|| ErrNO::NoMem)?;

                list.push_back(page);
            }
            i += 1;
        }

        pmm_node.add_free_pages(&mut list);
        dprint!(INFO, "init page_array ok!\n");
        Ok(())
    }
}
