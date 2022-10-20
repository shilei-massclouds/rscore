/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::*;
use crate::arch::defines::*;

pub struct BootAlloc {
    _start: usize,
    end: usize,
}

impl BootAlloc {
    pub fn new() -> BootAlloc {
        BootAlloc {
            _start: _end as usize,
            end: _end as usize,
        }
    }

    pub fn alloc_page_phys(&mut self) -> usize {
        let ptr = PAGE_ALIGN!(self.end);
        self.end = ptr + PAGE_SIZE;

        ptr
    }
}
