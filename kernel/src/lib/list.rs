/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

/*
 * A doubly-linked list with outside nodes.
 * The `LinkedList` allows pushing and popping elements
 * at either end in constant time.
 */

use core::mem;
use core::ptr::NonNull;
use core::marker::PhantomData;
use crate::vm::page::linked;

pub struct ListNode {
    next: Option<NonNull<ListNode>>,
    prev: Option<NonNull<ListNode>>,
}

impl ListNode {
    pub fn new() -> Self {
        ListNode {next: None, prev: None}
    }
}

pub struct List<T: linked> {
    head: Option<NonNull<ListNode>>,
    tail: Option<NonNull<ListNode>>,
    len: usize,
    marker: PhantomData<NonNull<T>>,
}

impl<T: linked> List<T> {
    /* Creates an empty `LinkedList`. */
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        List {
            head: None, tail: None, len: 0,
            marker: PhantomData
        }
    }

    /* Adds the given node to the back of the list. */
    #[inline]
    fn push_back_node(&mut self, node: &mut ListNode) {
        unsafe {
            node.next = None;
            node.prev = self.tail;
            let node = Some(node.into());

            match self.tail {
                None => self.head = node,
                Some(tail) => (*tail.as_ptr()).next = node,
            }

            self.tail = node;
            self.len += 1;
        }
    }

    pub fn push_back(&mut self, mut elt: NonNull<T>) {
        unsafe { self.push_back_node(elt.as_mut().into_node()); }
    }

    /* Removes and returns the node at the back of the list. */
    #[inline]
    fn pop_back_node(&mut self) -> Option<NonNull<ListNode>> {
        self.tail.map(|node| unsafe {
            let ptr = node.as_ptr();
            self.tail = (*ptr).prev;

            match self.tail {
                None => self.head = None,
                Some(tail) => (*tail.as_ptr()).next = None,
            }

            self.len -= 1;
            node
        })
    }

    pub fn pop_back(&mut self) -> Option<NonNull<T>> {
        T::from_node(self.pop_back_node()?)
    }

    pub fn append(&mut self, other: &mut Self) {
        match self.tail {
            None => mem::swap(self, other),
            Some(mut tail) => {
                /* `as_mut` is okay here because we have
                 * exclusive access to the entirety of both lists. */
                if let Some(mut other_head) = other.head.take() {
                    unsafe {
                        tail.as_mut().next = Some(other_head);
                        other_head.as_mut().prev = Some(tail);
                    }

                    self.tail = other.tail.take();
                    self.len += mem::replace(&mut other.len, 0);
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
