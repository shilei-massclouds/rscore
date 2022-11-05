/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::ptr::NonNull;
use alloc::string::String;
use crate::{
    vaddr_t, KERNEL_ASPACE_BASE, KERNEL_ASPACE_SIZE,
    dprint, INFO,
};
use crate::lib::list::{ListNode, Linked};

/* A representation of a contiguous range of virtual address space */
/* VmAddressRegionOrMapping */
struct VmAddressRegion {
}

impl VmAddressRegion {
    pub fn new() -> Self {
        Self {}
    }
}

pub enum VmAspaceType {
    User,
    Kernel,
    /* You probably do not want to use LOW_KERNEL. It is primarily used
     * for SMP bootstrap or mexec to allow mappings of very low memory
     * using the standard VMM subsystem. */
    LowKernel,
    /* an address space representing hypervisor guest memory */
    GuestPhysical,
}

pub struct VmAspace {
    queue_node: ListNode,
    name: String,
    base: vaddr_t,
    size: usize,
    as_type: VmAspaceType,
    root_vmar: Option<VmAddressRegion>,
}

impl Linked for VmAspace {
    fn from_node(ptr: NonNull<ListNode>) -> Option<NonNull<Self>> {
        NonNull::<Self>::new(ptr.as_ptr() as *mut Self)
    }

    fn into_node(&mut self) -> &mut ListNode {
        &mut (self.queue_node)
    }

    fn delete_from_list(&mut self) {
        self.into_node().delete_from_list();
    }
}

impl VmAspace {
    pub fn new(name: &str,
               base: vaddr_t,
               size: usize,
               as_type: VmAspaceType) -> Self {
        VmAspace {
            queue_node: ListNode::new(),
            name: String::from(name),
            base,
            size,
            as_type,
            root_vmar: None,
        }
    }
}

pub fn kernel_aspace_init_pre_heap() {
    let mut kernel_aspace =
        VmAspace::new("kernel", KERNEL_ASPACE_BASE, KERNEL_ASPACE_SIZE,
                      VmAspaceType::Kernel);

    let root_vmar = Some(VmAddressRegion::new());
    kernel_aspace.root_vmar = root_vmar;
    //kernel_aspace.init()?;

    //aspaces.push_front(kernel_aspace);
    dprint!(INFO, "kernel_aspace_init_pre_heap ok!\n");
}