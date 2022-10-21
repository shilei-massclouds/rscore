/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::config_generated::*;

pub const NR_CPUS: usize = _CONFIG_NR_CPUS;

pub const PAGE_SHIFT: usize = _CONFIG_PAGE_SHIFT;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
//pub const PAGE_MASK: usize = PAGE_SIZE - 1;

/* Virtual address where the kernel address space begins.
 * Below this is the user address space. */
pub const KERNEL_ASPACE_BASE: usize = 0xffff_0000_0000_0000;
//pub const KERNEL_ASPACE_SIZE: usize = 0x0001_0000_0000_0000;

/* map 512GB at the base of the kernel.
 * this is the max that can be mapped
 * with a single level 1 page table using 1GB pages.
 */
pub const ARCH_PHYSMAP_SIZE: usize = 1 << 39;

pub const KERNEL_BASE: usize = _CONFIG_KERNEL_BASE;

pub const SATP_MODE_48: usize = 0x9000000000000000;

/* These symbols come from kernel.ld */
extern "C" {
    pub fn __code_start();
    pub fn __bss_start();
    pub fn _end();
}
