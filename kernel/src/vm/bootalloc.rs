/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::*;
use crate::arch::defines::*;

/* Symbol _end comes from kernel.ld */
extern "C" {
    fn _end();
}

pub struct BootAlloc {
    start: usize,
    end: usize,
}

impl BootAlloc {
    pub fn new() -> BootAlloc {
        BootAlloc {
            start: _end as usize,
            end: _end as usize,
        }
    }

    pub fn alloc_page_phys(&mut self) -> usize {
        let ptr = ALIGN!(self.end, PAGE_SIZE);
        self.end = ptr + PAGE_SIZE;

        ptr
    }
}