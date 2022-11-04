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

    pub fn delete_from_list(&mut self) {
        if self.prev.is_none() || self.next.is_none() {
            return;
        }

        if let Some(next) = self.next {
            unsafe {(*next.as_ptr()).prev = self.prev.take();}
        }

        if let Some(prev) = self.prev {
            unsafe {(*prev.as_ptr()).next = self.next.take();}
        }
    }
}

pub struct List<T: linked> {
    node: ListNode,
    ref_node: Option<NonNull<ListNode>>,    /* ref to node */
    len: usize,
    marker: PhantomData<NonNull<T>>,
}

impl<T: linked> List<T> {
    /* Creates an empty `LinkedList`. */
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let mut list = List {
            node: ListNode::new(),
            ref_node: None,
            len: 0,
            marker: PhantomData
        };

        //list.node = NonNull::new(&mut list._node as *mut ListNode);
        list.ref_node = NonNull::new(&mut list.node);
        list.node.next = list.ref_node;
        list.node.prev = list.ref_node;

        /*
        let node = NonNull::new(&mut list.node as *mut ListNode);
        list.node.next = node;
        list.node.prev = node;
        */

        list
    }

    /* Adds the given node to the tail of the list. */
    #[inline]
    fn add_tail_node(&mut self, node: &mut ListNode) {
        node.prev = self.node.prev;
        node.next = self.ref_node;
        let node = Some(node.into());

        if let Some(prev) = self.node.prev {
            unsafe {(*prev.as_ptr()).next = node;}
        }
        self.node.prev = node;

        self.len += 1;
    }

    pub fn add_tail(&mut self, mut elt: NonNull<T>) {
        unsafe {self.add_tail_node(elt.as_mut().into_node());}
    }

    /* Removes and returns the node at the back of the list. */
    #[inline]
    fn remove_tail_node(&mut self) -> Option<NonNull<ListNode>> {
        if self.node.prev == self.ref_node {
            return None;
        }

        let node = self.node.prev;
        if let Some(mut prev) = self.node.prev {
            unsafe {prev.as_mut().delete_from_list();}
        }
        self.len -= 1;
        node
    }

    pub fn remove_tail(&mut self) -> Option<NonNull<T>> {
        T::from_node(self.remove_tail_node()?)
    }

    pub fn append(&mut self, other: &mut Self) {
        if other.node.prev == other.ref_node {
            return;
        }

        if let Some(next) = other.node.next {
            unsafe {(*next.as_ptr()).prev = self.node.prev;}
        }
        if let Some(prev) = other.node.prev {
            unsafe {(*prev.as_ptr()).next = self.ref_node;}
        }

        if let Some(prev) = self.node.prev {
            unsafe {(*prev.as_ptr()).next = other.node.next.take();}
        }
        self.node.prev = other.node.prev.take();

        self.len += mem::replace(&mut other.len, 0);
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
