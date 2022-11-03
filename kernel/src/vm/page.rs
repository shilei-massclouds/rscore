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
use crate::lib::list::ListNode;
use crate::vm::vm_page_state;
use crate::vm::vm_page_state::vm_page_state_t;

pub trait linked {
    fn into_node(&mut self) -> &mut ListNode;
    fn from_node(ptr: NonNull<ListNode>) -> Option<NonNull<Self>>;
}

pub struct vm_page {
    /* linked node */
    pub queue_node: ListNode,

    /* read-only after being set up */
    paddr_: paddr_t,  /* use paddr() accessor */

    /* offset 0x2b */

    /* logically private; use |state()| and |set_state()| */
    state_: AtomicU8,

    /* offset 0x2c */

}

impl linked for vm_page {
    fn into_node(&mut self) -> &mut ListNode {
        &mut (self.queue_node)
    }

    fn from_node(ptr: NonNull<ListNode>) -> Option<NonNull<Self>> {
        NonNull::<vm_page_t>::new(ptr.as_ptr() as *mut Self)
    }
}

impl vm_page {
    pub fn new() -> Self {
        vm_page {
            queue_node: ListNode::new(),
            paddr_: 0,
            state_: AtomicU8::new(vm_page_state::FREE),
        }
    }

    pub fn init(&mut self, paddr: paddr_t) {
        self.paddr_ = paddr;
    }

    pub fn set_state(&mut self, new_state: u8) {
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

    pub fn add_to_initial_count(state: vm_page_state_t, n: usize) {
        /*
        percpu::WithCurrentPreemptDisable(
            [&state, &n](percpu* p) {
                p->vm_page_counts.by_state[VmPageStateIndex(state)] +=
                    n;
        });
        */
    }
}

pub type vm_page_t = vm_page;
