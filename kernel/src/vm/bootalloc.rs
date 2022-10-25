/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::*;
use crate::arch::defines::*;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{
    AtomicUsize,
    Ordering::{Acquire, SeqCst},
};

const MAX_SUPPORTED_ALIGN: usize = 4096;

struct BootAllocator {
    allocated: AtomicUsize,
}

unsafe impl GlobalAlloc for BootAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        if align > MAX_SUPPORTED_ALIGN {
            return null_mut();
        }

        if self.allocated.fetch_update(SeqCst, SeqCst, |mut allocated| {
                allocated += size;
                allocated = ROUNDUP!(allocated, align);
                Some(allocated)
        }).is_err() {
            return null_mut();
        };

        (_end as *mut u8).add(self.allocated.load(Acquire))
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static ALLOCATOR: BootAllocator = BootAllocator {
    allocated: AtomicUsize::new(0),
};
