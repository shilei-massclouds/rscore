/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::vm::vm_aspace::kernel_aspace_init_pre_heap;

pub fn vm_init_preheap() {
    /* allow the vmm a shot at initializing some of its data structures */
    kernel_aspace_init_pre_heap();

    // vm_init_preheap_vmars();

    // // mark the physical pages used by the boot time allocator
    // if (boot_alloc_end != boot_alloc_start) {
    //   dprintf(INFO, "VM: marking boot alloc used range [%#" PRIxPTR ", %#" PRIxPTR ")\n",
    //           boot_alloc_start, boot_alloc_end);

    //   MarkPagesInUsePhys(boot_alloc_start, boot_alloc_end - boot_alloc_start);
    // }
}
