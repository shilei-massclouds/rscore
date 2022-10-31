/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

/*
 * The kernel physmap is a region of the kernel where all of
 * useful physical memory is mapped in one large chunk.
 * It's up to the individual architecture to decide how much
 * to map but it's usually a fairly large chunk at the base of
 * the kernel
 */

use crate::{KERNEL_ASPACE_BASE, ARCH_PHYSMAP_SIZE};

const PHYSMAP_BASE: usize = KERNEL_ASPACE_BASE;
const _PHYSMAP_SIZE: usize = ARCH_PHYSMAP_SIZE;
const PHYSMAP_BASE_PHYS: usize = 0;

/* physical to virtual in the big kernel map */
pub fn paddr_to_physmap(pa: usize) -> usize {
    pa - PHYSMAP_BASE_PHYS + PHYSMAP_BASE
}
