/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::config_generated::*;

pub const PAGE_SHIFT: usize = _CONFIG_PAGE_SHIFT;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

/* Virtual address where the kernel address space begins.
 * Below this is the user address space. */
pub const KERNEL_ASPACE_BASE: usize = 0xffff_0000_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0001_0000_0000_0000;

/* map 512GB at the base of the kernel.
 * this is the max that can be mapped
 * with a single level 1 page table using 1GB pages.
 */
pub const ARCH_PHYSMAP_SIZE: usize = 1 << 39;
