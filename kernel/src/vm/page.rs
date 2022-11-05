/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]

use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, Ordering};

use crate::paddr_t;
use crate::lib::list::{ListNode, Linked};
use crate::vm::vm_page_state;
use crate::vm::vm_page_state::vm_page_state_t;

const IsLoaned:         u8 = 1;
const IsLoanCancelled:  u8 = 2;

pub struct vm_page {
    /* linked node */
    pub queue_node: ListNode,

    /* read-only after being set up */
    paddr_: paddr_t,  /* use paddr() accessor */

    /* offset 0x2b */

    /* logically private; use |state()| and |set_state()| */
    state_: AtomicU8,

    /* offset 0x2c */

    /* logically private, use loaned getters and setters below. */
    loaned_state_: AtomicU8,
}

impl Linked for vm_page {
    fn from_node(ptr: NonNull<ListNode>) -> Option<NonNull<Self>> {
        NonNull::<vm_page_t>::new(ptr.as_ptr() as *mut Self)
    }

    fn into_node(&mut self) -> &mut ListNode {
        &mut (self.queue_node)
    }

    fn delete_from_list(&mut self) {
        self.into_node().delete_from_list();
    }
}

impl vm_page {
    pub fn new() -> Self {
        vm_page {
            queue_node: ListNode::new(),
            paddr_: 0,
            state_: AtomicU8::new(vm_page_state::FREE),
            loaned_state_: AtomicU8::new(0),
        }
    }

    pub fn init(&mut self, paddr: paddr_t) {
        self.paddr_ = paddr;
    }

    pub fn set_state(&mut self, new_state: vm_page_state_t) {
        let old_state = self.state_.swap(new_state, Ordering::Relaxed);

        /*
            auto& p = percpu::GetCurrent();
            p.vm_page_counts.by_state[VmPageStateIndex(old_state)] -= 1;
            p.vm_page_counts.by_state[VmPageStateIndex(new_state)] += 1;
        */
    }

    pub fn paddr(&self) -> paddr_t {
        self.paddr_
    }

    pub fn state(&self) -> vm_page_state_t {
        self.state_.load(Ordering::Relaxed)
    }

    pub fn add_to_initial_count(state: vm_page_state_t, n: usize) {
        /*
        percpu::WithCurrentPreemptDisable(
            [&state, &n](percpu* p) {
                p->vm_page_counts.by_state[VmPageStateIndex(state)] +=
                    n;
        });
        */
    }

    /* helper routines */

    /* Returns whether this page is in the FREE state.
     * When in the FREE state the page is assumed to be owned
     * by the relevant PmmNode, and hence unless its lock is
     * held this query must be assumed to be racy. */
    pub fn is_free(&self) -> bool {
        self.state() == vm_page_state::FREE
    }

    /* If true, this page is "loaned" in the sense of being loaned
     * from a contiguous VMO (via decommit) to Zircon.
     * If the original contiguous VMO is deleted, this page will
     * no longer be loaned. A loaned page cannot be pinned. Instead a different physical page (non-loaned) is used for the pin. A loaned page
     * can be (re-)committed back into its original contiguous VMO,
     * which causes the data in the loaned page to be moved into
     * a different physical page (which itself can be non-loaned or
     * loaned). A loaned page cannot be used to allocate
     * a new contiguous VMO. */
    pub fn is_loaned(&self) -> bool {
        (self.loaned_state_.load(Ordering::Relaxed) & IsLoaned)
            == IsLoaned
    }
}

pub type vm_page_t = vm_page;
